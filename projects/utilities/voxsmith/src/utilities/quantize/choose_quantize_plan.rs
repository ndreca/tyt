use crate::{
    Error, Result,
    utilities::{
        AlphaMode, ColorSpace, PartitionProperties, PropertyInterpretation, QuantizeOptions,
        QuantizePlan, QuantizePoint, ReductionMethod, kmeans, median_cut, octree,
        representative_point,
    },
};
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{
    FromColor, TyCielabColorF64, TyColorToVector3, TyLinSrgbF64, TyOklabColorF64, TySrgbF64,
    TyVector3F64, TyVector4F64,
};
use voxcore::{
    BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty, VoxExt, VoxMain, VoxPalette,
    VoxValuePool, VoxValuePoolKind, VoxValuePoolValueRef,
    material::{BASE_COLOR, EMISSIVE_COLOR},
};

/// Chooses representatives for the materials of `palette_id` that `layers`
/// sample, each weighted by its voxel count, or `None` when they sample at most
/// `options.max_materials`.
pub(crate) fn choose_quantize_plan<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    layers: &[(U32Id<BVoxObject>, U32Id<BVoxLayer>)],
    options: &QuantizeOptions,
) -> Result<Option<QuantizePlan>> {
    let palette = main
        .palette(palette_id)
        .expect("the quantized palette is one of the main's");
    let palette_index = main
        .iter_palettes()
        .position(|(listed_palette_id, _)| listed_palette_id == palette_id)
        .expect("the quantized palette is one of the main's");

    let name = &options.property;
    let property_id = palette.property_id_by_name(name).ok_or_else(|| {
        Error::invalid(format!("palette {palette_index} has no property `{name}`"))
    })?;
    let value_pool = property_value_pool(main, palette, property_id);

    let reading = Reading::resolve(options, value_pool.kind())?;
    let partition_properties =
        partition_properties(main, palette, palette_index, property_id, options)?;

    let populations = populations(main, palette_id, layers);
    let max_materials = options.max_materials.get();
    if populations.len() <= max_materials {
        return Ok(None);
    }

    // Candidates in listing order, grouped into partitions in order of first
    // appearance, so the clustering is deterministic.
    let mut partitions: Vec<(PartitionKey<'_>, Vec<QuantizePoint>)> = Vec::new();
    let mut partition_indices = HashMap::new();

    for (material_index, material_id) in palette.iter_materials().enumerate() {
        let Some(&population) = populations.get(&material_id) else {
            continue;
        };

        let value = material_value(palette, value_pool, material_id, property_id);
        let (coords, alpha) = reading.point(value).ok_or_else(|| {
            Error::invalid(format!(
                "material {material_index} of palette {palette_index} holds {value:?} for \
                 `{name}`, which reads as no finite point"
            ))
        })?;

        let key = PartitionKey {
            alpha,
            values: partition_properties
                .iter()
                .map(|&(partition_property_id, partition_value_pool)| {
                    material_value(
                        palette,
                        partition_value_pool,
                        material_id,
                        partition_property_id,
                    )
                })
                .collect(),
        };

        let point = QuantizePoint {
            material_id,
            coords,
            population,
        };

        let partition_index = match partitions.iter().position(|(other, _)| *other == key) {
            Some(partition_index) => partition_index,
            None => {
                partitions.push((key, Vec::new()));
                partitions.len() - 1
            }
        };
        partitions[partition_index].1.push(point);
        partition_indices.insert(material_id, partition_index);
    }

    if partitions.len() > max_materials {
        return Err(Error::invalid(format!(
            "the partitions split palette {palette_index}'s sampled materials into {} groups, \
             more than the {max_materials} material(s) allowed",
            partitions.len()
        )));
    }

    let partitions: Vec<Vec<QuantizePoint>> =
        partitions.into_iter().map(|(_, points)| points).collect();
    let partition_count = partitions.len();

    let clusters = match options.method {
        ReductionMethod::Kmeans => cluster_each_partition(partitions, max_materials, kmeans),
        ReductionMethod::MedianCut => median_cut(partitions, max_materials),
        ReductionMethod::Octree => cluster_each_partition(partitions, max_materials, octree),
    };

    let mut plan = QuantizePlan {
        representative_ids: HashMap::new(),
        points: HashMap::new(),
        partition_representatives: vec![Vec::new(); partition_count],
    };

    for cluster in clusters {
        let representative = representative_point(&cluster);
        let partition_index = partition_indices[&representative.material_id];
        plan.partition_representatives[partition_index].push(representative);

        for point in cluster {
            plan.representative_ids
                .insert(point.material_id, representative.material_id);
            plan.points
                .insert(point.material_id, (point, partition_index));
        }
    }

    Ok(Some(plan))
}

/// How a property's values read as clustering points, resolved against its
/// value pool's kind.
enum Reading {
    /// A color measured in `space`, alpha taking part per `alpha` when the
    /// color has four components.
    Color {
        encoding: ColorEncoding,
        space: ColorSpace,
        alpha: Option<AlphaMode>,
    },

    /// Raw components.
    Numeric,
}

/// Whether a color reading's components are linear light or sRGB-encoded.
#[derive(Clone, Copy)]
enum ColorEncoding {
    Linear,
    Srgb,
}

impl Reading {
    /// Resolves `options`'s reading of a property whose values are `kind`.
    fn resolve(options: &QuantizeOptions, kind: &VoxValuePoolKind) -> Result<Self> {
        let name = &options.property;
        let kind_name = kind_name(kind);

        let interpretation = match options.interpret_property {
            PropertyInterpretation::Auto if name == BASE_COLOR || name == EMISSIVE_COLOR => {
                PropertyInterpretation::LinearColor
            }

            PropertyInterpretation::Auto => PropertyInterpretation::Numeric,

            interpretation => interpretation,
        };

        let (reading, dimensions) = match interpretation {
            PropertyInterpretation::LinearColor | PropertyInterpretation::SrgbColor => {
                let components = match kind {
                    VoxValuePoolKind::Vec3Float(_) => 3,
                    VoxValuePoolKind::Vec4Float(_) => 4,
                    _ => {
                        return Err(Error::invalid(format!(
                            "property `{name}` holds {kind_name} values, but a color reading \
                             needs vec-3-float or vec-4-float"
                        )));
                    }
                };

                let alpha = match (components, options.alpha) {
                    (4, alpha) => Some(alpha.unwrap_or(AlphaMode::Partition)),
                    (_, None) => None,
                    (_, Some(_)) => {
                        return Err(Error::invalid(format!(
                            "property `{name}` reads as a 3-component color, which has no alpha \
                             to take part"
                        )));
                    }
                };

                let encoding = match interpretation {
                    PropertyInterpretation::SrgbColor => ColorEncoding::Srgb,
                    _ => ColorEncoding::Linear,
                };

                let dimensions = match alpha {
                    Some(AlphaMode::Distance) => 4,
                    _ => 3,
                };

                let reading = Reading::Color {
                    encoding,
                    space: options.space.unwrap_or(ColorSpace::Oklab),
                    alpha,
                };

                (reading, dimensions)
            }

            PropertyInterpretation::Numeric => {
                let dimensions = match kind {
                    VoxValuePoolKind::Float(_) | VoxValuePoolKind::Int(_) => 1,
                    VoxValuePoolKind::Vec2Float(_) | VoxValuePoolKind::Vec2Int(_) => 2,
                    VoxValuePoolKind::Vec3Float(_) | VoxValuePoolKind::Vec3Int(_) => 3,
                    VoxValuePoolKind::Vec4Float(_) | VoxValuePoolKind::Vec4Int(_) => 4,
                    VoxValuePoolKind::Bool(_)
                    | VoxValuePoolKind::Json(_)
                    | VoxValuePoolKind::String(_) => {
                        return Err(Error::invalid(format!(
                            "property `{name}` holds {kind_name} values, which read as no points"
                        )));
                    }
                };

                if options.alpha.is_some() {
                    return Err(Error::invalid(format!(
                        "property `{name}` reads as numeric, which has no alpha to take part"
                    )));
                }

                if options.space.is_some() {
                    return Err(Error::invalid(format!(
                        "property `{name}` reads as numeric, which a color space does not apply to"
                    )));
                }

                (Reading::Numeric, dimensions)
            }

            PropertyInterpretation::Auto => unreachable!("auto resolved above"),
        };

        if options.method == ReductionMethod::Octree && dimensions == 4 {
            return Err(Error::invalid(format!(
                "property `{name}` reads as 4D points, but octree clusters 3D points"
            )));
        }

        Ok(reading)
    }

    /// `value` as a clustering point, with its alpha when alpha partitions, or
    /// `None` when a coordinate is not finite. `value` has to be of the kind
    /// the reading resolved against.
    fn point(&self, value: VoxValuePoolValueRef<'_>) -> Option<(TyVector4F64, Option<f64>)> {
        let (coords, partition_alpha) = match *self {
            Reading::Color {
                encoding,
                space,
                alpha,
            } => {
                let (rgb, alpha_value) = match value {
                    VoxValuePoolValueRef::Vec3Float(&[red, green, blue]) => {
                        ([red, green, blue], None)
                    }
                    VoxValuePoolValueRef::Vec4Float(&[red, green, blue, alpha]) => {
                        ([red, green, blue], Some(alpha))
                    }
                    _ => unreachable!("a color reading resolved against a float vector"),
                };

                let xyz = color_coords(rgb, encoding, space);

                let (w, partition_alpha) = match (alpha, alpha_value) {
                    (Some(AlphaMode::Distance), Some(alpha)) => (alpha * alpha_scale(space), None),
                    (Some(AlphaMode::Partition), Some(alpha)) => (0.0, Some(alpha)),
                    _ => (0.0, None),
                };

                (xyz.extend(w), partition_alpha)
            }

            Reading::Numeric => {
                let mut components = [0.0; 4];
                match value {
                    VoxValuePoolValueRef::Float(number) => components[0] = number,
                    VoxValuePoolValueRef::Int(number) => components[0] = number as f64,
                    VoxValuePoolValueRef::Vec2Float(vector) => {
                        components[..2].copy_from_slice(vector)
                    }
                    VoxValuePoolValueRef::Vec3Float(vector) => {
                        components[..3].copy_from_slice(vector)
                    }
                    VoxValuePoolValueRef::Vec4Float(vector) => components.copy_from_slice(vector),
                    VoxValuePoolValueRef::Vec2Int(vector) => {
                        components[..2].copy_from_slice(&vector.map(|number| number as f64))
                    }
                    VoxValuePoolValueRef::Vec3Int(vector) => {
                        components[..3].copy_from_slice(&vector.map(|number| number as f64))
                    }
                    VoxValuePoolValueRef::Vec4Int(vector) => {
                        components.copy_from_slice(&vector.map(|number| number as f64))
                    }
                    _ => unreachable!("a numeric reading resolved against a number or vector"),
                }

                (TyVector4F64::from_array(components), None)
            }
        };

        coords.is_finite().then_some((coords, partition_alpha))
    }
}

/// The color `rgb`, stored in `encoding`, as a point in `space`.
fn color_coords(rgb: [f64; 3], encoding: ColorEncoding, space: ColorSpace) -> TyVector3F64 {
    let [red, green, blue] = rgb;

    let linear = match encoding {
        ColorEncoding::Linear => TyLinSrgbF64::new(red, green, blue),
        ColorEncoding::Srgb => TySrgbF64::new(red, green, blue).into_linear(),
    };

    match space {
        ColorSpace::Lab => TyCielabColorF64::from_color(linear).to_vector3(),
        ColorSpace::Oklab => TyOklabColorF64::from_color(linear).to_vector3(),
        ColorSpace::Srgb => match encoding {
            ColorEncoding::Linear => TySrgbF64::from_linear(linear).to_vector3(),
            ColorEncoding::Srgb => TyVector3F64::new(red, green, blue),
        },
    }
}

/// The factor bringing alpha's `[0, 1]` to the span of `space`'s lightness
/// axis, so a distance reading weighs alpha like lightness.
fn alpha_scale(space: ColorSpace) -> f64 {
    match space {
        ColorSpace::Lab => 100.0,
        ColorSpace::Oklab | ColorSpace::Srgb => 1.0,
    }
}

/// The partition key of one material: the values it has to share with another
/// material to merge.
#[derive(PartialEq)]
struct PartitionKey<'a> {
    alpha: Option<f64>,
    values: Vec<VoxValuePoolValueRef<'a>>,
}

/// The partition properties `options` picks from `palette`, each with the value
/// pool it draws from.
fn partition_properties<'a, T: VoxExt>(
    main: &'a VoxMain<T>,
    palette: &'a VoxPalette,
    palette_index: usize,
    property_id: U32Id<BVoxProperty>,
    options: &QuantizeOptions,
) -> Result<Vec<(U32Id<BVoxProperty>, &'a VoxValuePool)>> {
    let partition_property_ids: Vec<_> = match &options.partition {
        PartitionProperties::All => palette
            .iter_properties()
            .map(|(partition_property_id, _)| partition_property_id)
            .filter(|&partition_property_id| partition_property_id != property_id)
            .collect(),

        PartitionProperties::Named(names) => names
            .iter()
            .map(|partition_name| {
                if *partition_name == options.property {
                    return Err(Error::invalid(format!(
                        "property `{partition_name}` is the quantized property, so it cannot \
                         also partition"
                    )));
                }

                palette.property_id_by_name(partition_name).ok_or_else(|| {
                    Error::invalid(format!(
                        "palette {palette_index} has no property `{partition_name}` to \
                         partition on"
                    ))
                })
            })
            .collect::<Result<_>>()?,
    };

    Ok(partition_property_ids
        .into_iter()
        .map(|partition_property_id| {
            (
                partition_property_id,
                property_value_pool(main, palette, partition_property_id),
            )
        })
        .collect())
}

/// How many live voxels of `layers` sample each material of `palette_id`.
fn populations<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    layers: &[(U32Id<BVoxObject>, U32Id<BVoxLayer>)],
) -> HashMap<U32Id<BVoxMaterial>, u64> {
    let mut populations = HashMap::new();

    for &(object_id, layer_id) in layers {
        let object = main
            .object(object_id)
            .expect("a quantized layer's object is one of the main's");
        assert_eq!(
            object.layer_palette_id(layer_id),
            Some(palette_id),
            "a quantized layer references the quantized palette"
        );

        let samples = object
            .iter_live_samples(layer_id)
            .expect("a quantized layer is one of its object's");

        for (_, material_id) in samples {
            *populations.entry(material_id).or_insert(0) += 1;
        }
    }

    populations
}

/// Clusters each partition apart with `cluster`, into the slots
/// [`allocate_slots`] gives it.
fn cluster_each_partition(
    partitions: Vec<Vec<QuantizePoint>>,
    max_materials: usize,
    cluster: fn(Vec<QuantizePoint>, usize) -> Vec<Vec<QuantizePoint>>,
) -> Vec<Vec<QuantizePoint>> {
    let slots = allocate_slots(&partitions, max_materials);
    partitions
        .into_iter()
        .zip(slots)
        .flat_map(|(points, slot_count)| cluster(points, slot_count))
        .collect()
}

/// Hands each partition one slot, then the rest of `max_materials` one at a
/// time to the partition with the most voxels per slot, ties to the lowest
/// index. A partition takes at most one slot per point.
fn allocate_slots(partitions: &[Vec<QuantizePoint>], max_materials: usize) -> Vec<usize> {
    let populations: Vec<u128> = partitions
        .iter()
        .map(|points| points.iter().map(|point| point.population as u128).sum())
        .collect();

    let mut slots = vec![1usize; partitions.len()];

    for _ in partitions.len()..max_materials {
        let next = (0..partitions.len())
            .filter(|&index| slots[index] < partitions[index].len())
            .max_by(|&a, &b| {
                (populations[a] * slots[b] as u128)
                    .cmp(&(populations[b] * slots[a] as u128))
                    .then_with(|| b.cmp(&a))
            });

        let Some(index) = next else {
            break;
        };

        slots[index] += 1;
    }

    slots
}

/// The value pool `property_id` of `palette` draws from.
fn property_value_pool<'a, T: VoxExt>(
    main: &'a VoxMain<T>,
    palette: &VoxPalette,
    property_id: U32Id<BVoxProperty>,
) -> &'a VoxValuePool {
    let value_pool_id = palette
        .property(property_id)
        .expect("the property is one of the palette's")
        .value_pool_id;
    main.value_pool(value_pool_id)
        .expect("a property draws from a live value pool")
}

/// The value `material_id` holds for `property_id`, read from `value_pool`.
fn material_value<'a>(
    palette: &VoxPalette,
    value_pool: &'a VoxValuePool,
    material_id: U32Id<BVoxMaterial>,
    property_id: U32Id<BVoxProperty>,
) -> VoxValuePoolValueRef<'a> {
    let value_id = palette
        .value_id(material_id, property_id)
        .expect("a live material holds a value for every property");
    value_pool
        .value(value_id)
        .expect("a material's value is one of its value pool's")
}

/// The voxj type name of `kind`'s values.
fn kind_name(kind: &VoxValuePoolKind) -> &'static str {
    match kind {
        VoxValuePoolKind::Bool(_) => "bool",
        VoxValuePoolKind::Float(_) => "float",
        VoxValuePoolKind::Int(_) => "int",
        VoxValuePoolKind::Json(_) => "json",
        VoxValuePoolKind::String(_) => "string",
        VoxValuePoolKind::Vec2Float(_) => "vec-2-float",
        VoxValuePoolKind::Vec2Int(_) => "vec-2-int",
        VoxValuePoolKind::Vec3Float(_) => "vec-3-float",
        VoxValuePoolKind::Vec3Int(_) => "vec-3-int",
        VoxValuePoolKind::Vec4Float(_) => "vec-4-float",
        VoxValuePoolKind::Vec4Int(_) => "vec-4-int",
    }
}

#[cfg(test)]
mod tests {
    use super::allocate_slots;
    use crate::utilities::QuantizePoint;
    use branded_id::U32Id;
    use ty_math::TyVector4F64;

    /// A partition of one point per population.
    fn partition(populations: &[u64]) -> Vec<QuantizePoint> {
        populations
            .iter()
            .enumerate()
            .map(|(index, &population)| QuantizePoint {
                material_id: U32Id::from_u32(index as u32),
                coords: TyVector4F64::ZERO,
                population,
            })
            .collect()
    }

    #[test]
    fn slots_follow_voxel_counts_after_one_each() {
        let partitions = [partition(&[30, 30, 30]), partition(&[5, 5])];
        assert_eq!(allocate_slots(&partitions, 4), [3, 1]);
    }

    #[test]
    fn a_partition_takes_at_most_one_slot_per_point() {
        let partitions = [partition(&[100]), partition(&[1, 1, 1])];
        assert_eq!(allocate_slots(&partitions, 3), [1, 2]);
        assert_eq!(allocate_slots(&partitions, 10), [1, 3]);
    }

    #[test]
    fn ties_go_to_the_lower_partition() {
        let partitions = [partition(&[5, 5]), partition(&[5, 5])];
        assert_eq!(allocate_slots(&partitions, 3), [2, 1]);
    }
}
