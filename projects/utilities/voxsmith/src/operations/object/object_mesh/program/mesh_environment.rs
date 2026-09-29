use crate::{
    Error, Result,
    operations::object::{
        Computation, ComputedBinding, MeshElement, MeshGeometry, Swatches, is_solid,
    },
};
use branded_id::{U32Id, UsizeId};
use std::{collections::HashMap, slice};
use vox_value_language::{
    Components, Dimension, Domain, Groupings, TypeEnvironment, Value, ValueEnvironment,
};
use voxcore::{VoxObject, VoxValuePoolKind, VoxValuePoolValueRef};

/// The names a run's program reads but never defines: the effective palette's
/// properties, one swatch array each, and the computed bindings.
pub struct MeshEnvironment {
    pub types: TypeEnvironment,

    pub values: ValueEnvironment,
}

impl MeshEnvironment {
    /// Binds the properties `swatches` read through and `computed_bindings`
    /// over `object` and `geometry`. Errors on a computed binding shadowing
    /// a property or bound twice.
    pub(crate) fn bind(
        object: &VoxObject,
        swatches: &Swatches<'_>,
        geometry: &MeshGeometry,
        computed_bindings: &[ComputedBinding],
    ) -> Result<Self> {
        let mut types = HashMap::new();
        let mut values = HashMap::new();

        let effective = swatches.effective();

        for index in 0..effective.property_count() {
            let property_id = UsizeId::from_usize(index);

            let property = effective
                .property(property_id)
                .expect("property ids below the count resolve");

            let swatch_values: Vec<_> = (0..swatches.count())
                .map(|swatch| swatches.value(U32Id::from_u32(swatch as u32), property_id))
                .collect();

            if let Some(value) = property_value(
                property.name(),
                property.value_pool().kind(),
                &swatch_values,
            )? {
                types.insert(property.name().to_owned(), value.to_type());
                values.insert(property.name().to_owned(), value);
            }
        }

        let entries = |domain: Domain| match domain {
            Domain::Corner => geometry.quad_count() * 4,
            Domain::Face => geometry.quad_count(),
            Domain::Plain => unreachable!("a computed index runs over an array domain"),
            Domain::Swatch => swatches.count(),
            Domain::Voxel => swatches.voxel_swatch_ids().len(),
        };

        for binding in computed_bindings {
            let element = MeshElement::ComputedBinding {
                name: binding.name.clone(),
            };

            if effective.property_id_by_name(&binding.name).is_some() {
                return Err(Error::mesh_record(
                    element,
                    "shadows the palette property of the same name",
                ));
            }

            if values.contains_key(&binding.name) {
                return Err(Error::mesh_record(element, "is bound twice"));
            }

            let value = match binding.computation {
                Computation::Index(domain) => {
                    let domain = Domain::from(domain);
                    compute_index(domain, entries(domain))
                }

                Computation::Occlusion => compute_occlusion(object, geometry),

                Computation::VoxelPosition => compute_voxel_position(object),
            };

            types.insert(binding.name.clone(), value.to_type());
            values.insert(binding.name.clone(), value);
        }

        Ok(MeshEnvironment {
            types: TypeEnvironment { types },
            values: ValueEnvironment {
                values,
                groupings: groupings_of(swatches, geometry),
            },
        })
    }
}

/// The swatch array of the property `name`, whose pool has `kind`, from its
/// per-swatch `values`, or `None` for a json property, which the language
/// has no type for. An int reads as `u32` and errors outside its range.
fn property_value(
    name: &str,
    kind: &VoxValuePoolKind,
    values: &[VoxValuePoolValueRef<'_>],
) -> Result<Option<Value>> {
    let (dimension, components) = match kind {
        VoxValuePoolKind::Bool(_) => (
            Dimension::Vec1,
            Components::Bool(
                values
                    .iter()
                    .map(|value| match value {
                        VoxValuePoolValueRef::Bool(value) => *value,
                        _ => unreachable!("a pool's values share its kind"),
                    })
                    .collect(),
            ),
        ),

        VoxValuePoolKind::Float(_) => (
            Dimension::Vec1,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Float(value) => slice::from_ref(value),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Int(_) => (
            Dimension::Vec1,
            u32s(name, values, 1, |value| match value {
                VoxValuePoolValueRef::Int(value) => slice::from_ref(value),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Json(_) => return Ok(None),

        VoxValuePoolKind::String(_) => (
            Dimension::Vec1,
            Components::String(
                values
                    .iter()
                    .map(|value| match value {
                        VoxValuePoolValueRef::String(value) => (*value).to_owned(),
                        _ => unreachable!("a pool's values share its kind"),
                    })
                    .collect(),
            ),
        ),

        VoxValuePoolKind::Vec2Float(_) => (
            Dimension::Vec2,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Vec2Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec2Int(_) => (
            Dimension::Vec2,
            u32s(name, values, 2, |value| match value {
                VoxValuePoolValueRef::Vec2Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Vec3Float(_) => (
            Dimension::Vec3,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Vec3Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec3Int(_) => (
            Dimension::Vec3,
            u32s(name, values, 3, |value| match value {
                VoxValuePoolValueRef::Vec3Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Vec4Float(_) => (
            Dimension::Vec4,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Vec4Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec4Int(_) => (
            Dimension::Vec4,
            u32s(name, values, 4, |value| match value {
                VoxValuePoolValueRef::Vec4Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),
    };

    let value = Value::new(Domain::Swatch, dimension, components)
        .expect("every swatch contributes one entry of the pool's width");

    Ok(Some(value))
}

/// The float components of `values` flattened, each read through `pick`.
fn f32s<'a>(
    values: &[VoxValuePoolValueRef<'a>],
    pick: for<'b> fn(&'b VoxValuePoolValueRef<'a>) -> &'b [f64],
) -> Components {
    Components::F32(
        values
            .iter()
            .flat_map(|value| pick(value).iter().map(|&component| component as f32))
            .collect(),
    )
}

/// The int components of `values` flattened as `u32`, each read through
/// `pick`, erroring on one outside the range and reporting its swatch.
fn u32s<'a>(
    name: &str,
    values: &[VoxValuePoolValueRef<'a>],
    width: usize,
    pick: for<'b> fn(&'b VoxValuePoolValueRef<'a>) -> &'b [i64],
) -> Result<Components> {
    let mut components = Vec::with_capacity(values.len() * width);

    for (swatch, value) in (0..).zip(values) {
        for &component in pick(value) {
            let component = u32::try_from(component).map_err(|_| {
                Error::invalid(format!(
                    "the property `{name}` holds {component} at swatch {swatch}, and the value \
                     language reads an int as u32"
                ))
            })?;

            components.push(component);
        }
    }

    Ok(Components::U32(components))
}

/// Each entry's index in `domain`, a `u32` vec1 array of `count` entries
/// counting up from `0`.
fn compute_index(domain: Domain, count: usize) -> Value {
    Value::new(
        domain,
        Dimension::Vec1,
        Components::U32((0..count).map(|index| index as u32).collect()),
    )
    .expect("one component per entry fills a vec1 array")
}

/// Each live voxel's grid position from the minimum corner of the live
/// extent, a voxel `u32` vec3 array in raster order.
fn compute_voxel_position(object: &VoxObject) -> Value {
    let origin = object
        .live_extent()
        .map(|(minimum, _)| minimum.to_array())
        .unwrap_or([0; 3]);

    let mut components = Vec::with_capacity(object.live_count() * 3);

    for voxel_id in object.iter_live() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid")
            .to_array();

        components.extend((0..3).map(|axis| position[axis] - origin[axis]));
    }

    Value::new(Domain::Voxel, Dimension::Vec3, Components::U32(components))
        .expect("three components per voxel fill a vec3 array")
}

/// Each face corner's occlusion from the voxels meeting there, a corner `f32`
/// vec1 array in `[0, 1]` with `1` fully open. Each of the three cells beside
/// the corner in the layer the face looks into closes a third. Both cells
/// along the face's edges together close it fully. So does a solid cell over
/// the face, which only `naive` emits.
fn compute_occlusion(object: &VoxObject, geometry: &MeshGeometry) -> Value {
    let mut components = Vec::with_capacity(geometry.positions.len());

    for quad in 0..geometry.quad_count() {
        let corners: Vec<[f32; 3]> = geometry.positions[quad * 4..quad * 4 + 4]
            .iter()
            .map(|position| position.to_array())
            .collect();

        let normal = geometry.normals[quad * 4].to_array();

        let axis = (0..3)
            .find(|&axis| normal[axis] != 0.0)
            .expect("a face normal lies along one axis");

        let [first, second] = match axis {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        };

        let center = |axis: usize| corners.iter().map(|corner| corner[axis]).sum::<f32>() / 4.0;
        let centers = [center(first), center(second)];

        for corner in &corners {
            let mut cell = [0i64; 3];
            cell[axis] = if normal[axis] > 0.0 {
                corner[axis] as i64
            } else {
                corner[axis] as i64 - 1
            };

            // Along each tangent axis, the cell under the face and the cell
            // beside the corner outside it.
            let along = |tangent: usize, center: f32| {
                let at = corner[tangent] as i64;
                if corner[tangent] < center {
                    (at, at - 1)
                } else {
                    (at - 1, at)
                }
            };
            let (under_first, beside_first) = along(first, centers[0]);
            let (under_second, beside_second) = along(second, centers[1]);

            let solid = |first_at: i64, second_at: i64| {
                let mut cell = cell;
                cell[first] = first_at;
                cell[second] = second_at;
                is_solid(object, cell)
            };

            let over = solid(under_first, under_second);
            let side_first = solid(beside_first, under_second);
            let side_second = solid(under_first, beside_second);
            let diagonal = solid(beside_first, beside_second);

            let open = if over || (side_first && side_second) {
                0
            } else {
                3 - u8::from(side_first) - u8::from(side_second) - u8::from(diagonal)
            };

            components.push(f32::from(open) / 3.0);
        }
    }

    Value::new(Domain::Corner, Dimension::Vec1, Components::F32(components))
        .expect("one component per corner fills a vec1 array")
}

/// The tables the reductions and climbs walk: each voxel entry's swatch and
/// each face's voxel pieces.
fn groupings_of(swatches: &Swatches<'_>, geometry: &MeshGeometry) -> Groupings {
    Groupings {
        voxel_swatches: swatches.voxel_swatch_ids().clone(),
        face_voxels: geometry
            .face_voxel_ids
            .iter()
            .map(|voxel_ids| {
                voxel_ids
                    .iter()
                    .map(|&voxel_id| swatches.voxel_entry_id(voxel_id))
                    .collect()
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::{
            Method, Swatches,
            object_mesh::program::mesh_environment::{
                compute_index, compute_occlusion, compute_voxel_position, groupings_of,
                property_value,
            },
            object_to_mesh_geometry,
        },
        test_utilities::live_object,
    };
    use vox_value_language::{Components, Dimension, Domain, Scalar};
    use voxcore::{VoxMain, VoxObject, VoxValue, VoxValuePool, VoxValuePoolValueRef};

    #[test]
    fn the_index_counts_the_entries_up() {
        let value = compute_index(Domain::Face, 3);

        assert_eq!(value.domain(), Domain::Face);
        assert_eq!(value.components(), &Components::U32(vec![0, 1, 2]));
    }

    #[test]
    fn positions_start_at_the_live_extent_in_raster_order() {
        let object = live_object([4, 4, 4], &[[3, 2, 1], [1, 2, 1], [1, 3, 2]]);

        let value = compute_voxel_position(&object);

        assert_eq!(value.domain(), Domain::Voxel);
        assert_eq!(
            value.components(),
            &Components::U32(vec![0, 0, 0, 0, 1, 1, 2, 0, 0])
        );
    }

    /// The occlusion at each corner of the quads facing `normal`, as
    /// `(corner position, occlusion)`.
    fn facing(object: &VoxObject, method: Method, normal: [f32; 3]) -> Vec<([f32; 3], f32)> {
        let geometry = object_to_mesh_geometry(object, method);
        let value = compute_occlusion(object, &geometry);
        let Components::F32(occlusion) = value.components() else {
            panic!("occlusion is f32");
        };

        (0..geometry.quad_count())
            .filter(|&quad| geometry.normals[quad * 4].to_array() == normal)
            .flat_map(|quad| {
                (quad * 4..quad * 4 + 4)
                    .map(|corner| (geometry.positions[corner].to_array(), occlusion[corner]))
            })
            .collect()
    }

    #[test]
    fn a_lone_voxel_is_open_at_every_corner() {
        let object = live_object([3, 3, 3], &[[1, 1, 1]]);
        let geometry = object_to_mesh_geometry(&object, Method::Culled);

        let value = compute_occlusion(&object, &geometry);

        assert_eq!(value.entries(), 24);
        assert_eq!(value.components(), &Components::F32(vec![1.0; 24]));
    }

    #[test]
    fn a_neighbor_beside_a_corner_closes_a_third_and_two_close_it() {
        // A step: the top of (0,0,0) meets (1,0,1) along its x = 1 edge.
        let step = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 0], [1, 0, 1]]);
        for (position, occlusion) in facing(&step, Method::Culled, [0.0, 0.0, 1.0]) {
            if position[2] != 1.0 {
                continue;
            }
            let expected = if position[0] == 1.0 { 2.0 / 3.0 } else { 1.0 };
            assert_eq!(occlusion, expected, "{position:?}");
        }

        // An inner corner: the top of (0,0,0) meets both (1,0,1) and (0,1,1),
        // and the diagonal (1,1,1) is empty.
        let inner = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 1], [0, 1, 1]]);
        let corners = facing(&inner, Method::Culled, [0.0, 0.0, 1.0]);
        let (_, occlusion) = corners
            .iter()
            .find(|(position, _)| *position == [1.0, 1.0, 1.0])
            .unwrap();
        assert_eq!(*occlusion, 0.0);
    }

    #[test]
    fn a_face_under_a_solid_cell_is_closed() {
        // Under naive the shared face between two voxels still emits.
        let pair = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 0]]);
        let corners = facing(&pair, Method::Naive, [1.0, 0.0, 0.0]);
        let buried: Vec<f32> = corners
            .iter()
            .filter(|(position, _)| position[0] == 1.0)
            .map(|(_, occlusion)| *occlusion)
            .collect();
        assert_eq!(buried, [0.0; 4]);
    }

    #[test]
    fn a_merged_face_lists_every_voxel_it_covers() {
        let main: VoxMain = VoxMain::default();
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let geometry = object_to_mesh_geometry(&object, Method::Greedy);

        let groupings = groupings_of(&swatches, &geometry);

        assert_eq!(groupings.voxel_swatches.len(), 2);
        assert_eq!(groupings.face_voxels.len(), 6);
        let mut pieces: Vec<usize> = groupings
            .face_voxels
            .iter()
            .map(|voxel_ids| voxel_ids.len())
            .collect();
        pieces.sort_unstable();
        assert_eq!(pieces, [1, 1, 2, 2, 2, 2]);
    }

    #[test]
    fn each_pool_kind_binds_its_language_type() {
        let cases: [(VoxValuePool, Dimension, Scalar, usize); 6] = [
            (
                VoxValuePool::boolean(vec![true]),
                Dimension::Vec1,
                Scalar::Bool,
                1,
            ),
            (
                VoxValuePool::float(vec![0.5]).unwrap(),
                Dimension::Vec1,
                Scalar::F32,
                1,
            ),
            (
                VoxValuePool::int(vec![7]).unwrap(),
                Dimension::Vec1,
                Scalar::U32,
                1,
            ),
            (
                VoxValuePool::string(vec!["glass".to_owned()]),
                Dimension::Vec1,
                Scalar::String,
                1,
            ),
            (
                VoxValuePool::vec_3_int(vec![[1, 2, 3]]).unwrap(),
                Dimension::Vec3,
                Scalar::U32,
                3,
            ),
            (
                VoxValuePool::vec_4_float(vec![[0.0, 0.5, 1.0, 1.0]]).unwrap(),
                Dimension::Vec4,
                Scalar::F32,
                4,
            ),
        ];

        for (pool, dimension, scalar, components) in cases {
            let values: Vec<VoxValuePoolValueRef> =
                pool.iter_values().map(|(_, value)| value).collect();

            let value = property_value("p", pool.kind(), &values).unwrap().unwrap();

            assert_eq!(value.domain(), Domain::Swatch, "{dimension} {scalar}");
            assert_eq!(value.dimension(), dimension, "{dimension} {scalar}");
            assert_eq!(value.scalar(), scalar, "{dimension} {scalar}");
            assert_eq!(value.components().len(), components, "{dimension} {scalar}");
        }
    }

    #[test]
    fn swatches_flatten_in_order() {
        let pool = VoxValuePool::vec_2_float(vec![[1.0, 2.0], [3.0, 4.0]]).unwrap();
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        let value = property_value("p", pool.kind(), &values).unwrap().unwrap();

        assert_eq!(value.entries(), 2);
        assert_eq!(
            value.components(),
            &Components::F32(vec![1.0, 2.0, 3.0, 4.0])
        );
    }

    #[test]
    fn an_int_outside_u32_errors_at_its_swatch() {
        let pool = VoxValuePool::int(vec![1, -1]).unwrap();
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        let error = property_value("tag", pool.kind(), &values)
            .unwrap_err()
            .to_string();

        assert!(error.contains("`tag` holds -1 at swatch 1"), "{error}");
    }

    #[test]
    fn a_json_property_binds_nothing() {
        let pool = VoxValuePool::json(vec![VoxValue::Null]);
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        assert_eq!(property_value("meta", pool.kind(), &values).unwrap(), None);
    }
}
