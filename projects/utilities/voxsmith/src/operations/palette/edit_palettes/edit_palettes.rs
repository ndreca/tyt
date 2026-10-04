use crate::{
    Error, Result,
    operations::palette::{PaletteEditElement, PropertyWrite},
    utilities::{WrittenValues, property_value, release_undrawn_values},
};
use branded_id::U32Id;
use std::collections::{BTreeSet, HashMap};
use vox_value_language::{
    Expression, Groupings, Program, TypeEnvironment, ValueEnvironment, check, check_expression,
    eval, eval_expression, parse, parse_expression,
};
use voxcore::{
    BVoxPalette, BVoxProperty, BVoxValuePool, BVoxValuePoolValue, Error as VoxError, VoxExt,
    VoxMain, VoxPalette,
};

/// One write evaluated over one palette, ready to land.
struct PlannedWrite {
    property: String,

    /// `None` when the palette lacks the property.
    property_id: Option<U32Id<BVoxProperty>>,

    values: WrittenValues,
}

/// Runs `program` over each palette of `palette_ids` and lands each of
/// `writes` in its property. Each property the program or a write reads
/// enters as a swatch array holding one entry per material. A property the
/// palette lacks is added on a new value pool. Every palette evaluates
/// against the unchanged main before any write lands. A value no palette
/// draws afterward is released. Errors, changing nothing, if:
///
/// 1. an id is not one of the main's palettes
/// 2. two writes write one property
/// 3. the program or a write fails to parse, check, or evaluate over a palette
/// 4. a write evaluates to a voxel, face, or corner value, or to a value no
///    value pool kind holds
/// 5. a write's kind differs from its property's value pool's
/// 6. a write puts a vocabulary property outside its range
pub fn edit_palettes<T: VoxExt>(
    main: &mut VoxMain<T>,
    palette_ids: &[U32Id<BVoxPalette>],
    program: &str,
    writes: &[PropertyWrite],
) -> Result<()> {
    let program =
        parse(program).map_err(|error| Error::palette_edit(PaletteEditElement::Program, error))?;

    let mut expressions = Vec::with_capacity(writes.len());

    for (index, write) in writes.iter().enumerate() {
        let element = write_element(write);

        if writes[..index]
            .iter()
            .any(|earlier| earlier.property == write.property)
        {
            return Err(Error::palette_edit(element, "repeats"));
        }

        let expression = parse_expression(&write.expression)
            .map_err(|error| Error::palette_edit(element, error))?;

        expressions.push(expression);
    }

    let free_names = program.free_names(&expressions);

    let mut planned = Vec::with_capacity(palette_ids.len());

    for &palette_id in palette_ids {
        let Some(palette) = main.palette(palette_id) else {
            return Err(VoxError::UnknownPalette { palette_id }.into());
        };

        let palette_writes =
            plan_writes(main, palette, &program, writes, &expressions, &free_names)
                .map_err(|error| Error::in_palette(palette_id, error))?;

        planned.push((palette_id, palette_writes));
    }

    let mut replaced_value_ids = Vec::new();

    for (palette_id, palette_writes) in planned {
        for write in palette_writes {
            match write.property_id {
                Some(property_id) => {
                    replaced_value_ids.extend(overwrite_property(
                        main,
                        palette_id,
                        property_id,
                        write.values,
                    ));
                }

                None => add_property(main, palette_id, write.property, write.values),
            }
        }
    }

    release_undrawn_values(main, replaced_value_ids)
}

fn write_element(write: &PropertyWrite) -> PaletteEditElement {
    PaletteEditElement::Write {
        property: write.property.clone(),
    }
}

/// Reports any check error of `program` or `writes` before any evaluation
/// error.
fn plan_writes<T: VoxExt>(
    main: &VoxMain<T>,
    palette: &VoxPalette,
    program: &Program,
    writes: &[PropertyWrite],
    expressions: &[Expression],
    free_names: &BTreeSet<String>,
) -> Result<Vec<PlannedWrite>> {
    let (types, values) = bind_properties(main, palette, free_names)?;

    let checked = check(program.clone(), &types)
        .map_err(|error| Error::palette_edit(PaletteEditElement::Program, error))?;

    let mut checked_expressions = Vec::with_capacity(writes.len());

    for (write, expression) in writes.iter().zip(expressions) {
        let checked_expression = check_expression(expression, &checked)
            .map_err(|error| Error::palette_edit(write_element(write), error))?;

        checked_expressions.push(checked_expression);
    }

    let evaluated = eval(&checked, &values)
        .map_err(|error| Error::palette_edit(PaletteEditElement::Program, error))?;

    let mut planned = Vec::with_capacity(writes.len());

    for (write, expression) in writes.iter().zip(&checked_expressions) {
        let value = eval_expression(expression, &evaluated)
            .map_err(|error| Error::palette_edit(write_element(write), error))?;

        let values = WrittenValues::of(value, palette.material_count())
            .map_err(|reason| Error::palette_edit(write_element(write), reason))?;

        let property_id = palette.property_id_by_name(&write.property);

        if let Some(property_id) = property_id {
            let property = palette
                .property(property_id)
                .expect("a resolved name identifies one of the palette's properties");

            let value_pool = main
                .value_pool(property.value_pool_id)
                .expect("a property names a live value pool");

            values
                .check_fits(value_pool)
                .map_err(|reason| Error::palette_edit(write_element(write), reason))?;
        }

        values.check_range(&write.property)?;

        planned.push(PlannedWrite {
            property: write.property.clone(),
            property_id,
            values,
        });
    }

    Ok(planned)
}

/// The types and values of each property of `palette` that `free_names`
/// holds, over one swatch per material. A json property stays unbound.
fn bind_properties<T: VoxExt>(
    main: &VoxMain<T>,
    palette: &VoxPalette,
    free_names: &BTreeSet<String>,
) -> Result<(TypeEnvironment, ValueEnvironment)> {
    let mut types = HashMap::new();
    let mut values = HashMap::new();

    for (property_id, property) in palette.iter_properties() {
        if !free_names.contains(&property.name) {
            continue;
        }

        let value_pool = main
            .value_pool(property.value_pool_id)
            .expect("a property names a live value pool");

        let value_ids: Vec<_> = palette
            .iter_materials()
            .map(|material_id| {
                palette
                    .value_id(material_id, property_id)
                    .expect("a live material holds a value for every property")
            })
            .collect();

        let Some(value) = property_value(&property.name, value_pool, &value_ids)? else {
            continue;
        };

        types.insert(property.name.clone(), value.to_type());
        values.insert(property.name.clone(), value);
    }

    let groupings = Groupings {
        swatch_count: palette.material_count(),
        ..Groupings::default()
    };

    Ok((
        TypeEnvironment { types },
        ValueEnvironment { values, groupings },
    ))
}

/// Lands `values` in palette `palette_id`'s property `property_id`, returning
/// the values its materials drew before.
fn overwrite_property<T: VoxExt>(
    main: &mut VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    property_id: U32Id<BVoxProperty>,
    values: WrittenValues,
) -> Vec<(U32Id<BVoxValuePool>, U32Id<BVoxValuePoolValue>)> {
    let palette = main
        .palette(palette_id)
        .expect("a planned palette is one of the main's");

    let value_pool_id = palette
        .property(property_id)
        .expect("a planned property is one of its palette's")
        .value_pool_id;

    let material_ids: Vec<_> = palette.iter_materials().collect();

    let replaced_value_ids = material_ids
        .iter()
        .map(|&material_id| {
            let value_id = palette
                .value_id(material_id, property_id)
                .expect("a live material holds a value for every property");

            (value_pool_id, value_id)
        })
        .collect();

    let value_ids = values.land(main, value_pool_id);

    for (material_id, value_id) in material_ids.into_iter().zip(value_ids) {
        main.set_material_value(palette_id, material_id, property_id, value_id)
            .expect("a landed value is one of its property's value pool's");
    }

    replaced_value_ids
}

/// Adds `property` to palette `palette_id` on a new value pool holding
/// `values`.
fn add_property<T: VoxExt>(
    main: &mut VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    property: String,
    values: WrittenValues,
) {
    let value_pool_id = main.retain_value_pool(values.empty_value_pool());

    let value_ids = values.land(main, value_pool_id);

    let Some(&default_value_id) = value_ids.first() else {
        main.retain_property(palette_id, property, value_pool_id)
            .expect("a planned property is new to its materialless palette");

        return;
    };

    let property_id = main
        .retain_property_filled(palette_id, property, value_pool_id, default_value_id)
        .expect("a planned property is new to its palette");

    let material_ids: Vec<_> = main
        .palette(palette_id)
        .expect("a planned palette is one of the main's")
        .iter_materials()
        .collect();

    for (material_id, value_id) in material_ids.into_iter().zip(value_ids) {
        main.set_material_value(palette_id, material_id, property_id, value_id)
            .expect("a landed value is one of its property's value pool's");
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Error,
        operations::palette::{PaletteEditElement, PropertyWrite, edit_palettes},
    };
    use branded_id::U32Id;
    use voxcore::{
        BVoxPalette, VoxMain, VoxPalette, VoxValue, VoxValueColumn, VoxValuePool,
        material::ROUGHNESS,
    };

    /// Retains a palette of `properties`, each on its own value pool, with one
    /// material per `rows` entry picking a value index per property.
    fn retain_palette(
        main: &mut VoxMain,
        properties: Vec<(&str, VoxValuePool)>,
        rows: &[Vec<u32>],
    ) -> U32Id<BVoxPalette> {
        let mut palette = VoxPalette::default();

        for (name, value_pool) in properties {
            let value_pool_id = main.retain_value_pool(value_pool);

            palette
                .retain_property(name.to_owned(), value_pool_id)
                .unwrap();
        }

        for row in rows {
            palette
                .retain_material(row.iter().map(|&index| U32Id::from_u32(index)).collect())
                .unwrap();
        }

        main.retain_palette(palette).unwrap()
    }

    /// A palette whose material `i` carries `tags[i]` and `roughness[i]`.
    fn retain_tagged(main: &mut VoxMain, tags: &[&str], roughness: &[f64]) -> U32Id<BVoxPalette> {
        retain_palette(
            main,
            vec![
                (
                    "tag",
                    VoxValuePool::string(tags.iter().map(|&tag| tag.to_owned()).collect()),
                ),
                (ROUGHNESS, VoxValuePool::float(roughness.to_vec()).unwrap()),
            ],
            &(0..tags.len() as u32)
                .map(|index| vec![index, index])
                .collect::<Vec<_>>(),
        )
    }

    /// The values the materials of `palette_id` draw for `name`, in material
    /// order.
    fn drawn<'a, V: Clone>(
        main: &'a VoxMain,
        palette_id: U32Id<BVoxPalette>,
        name: &str,
        column: fn(&'a VoxValuePool) -> Option<VoxValueColumn<'a, V>>,
    ) -> Vec<V> {
        let palette = main.palette(palette_id).unwrap();
        let property_id = palette.property_id_by_name(name).unwrap();

        palette
            .iter_materials()
            .map(|material_id| {
                let (value_pool, value_id) = main
                    .material_value(palette_id, material_id, property_id)
                    .unwrap();

                column(value_pool).unwrap().get(value_id).unwrap().clone()
            })
            .collect()
    }

    /// The number of values in the value pool `name` draws from in
    /// `palette_id`.
    fn value_count(main: &VoxMain, palette_id: U32Id<BVoxPalette>, name: &str) -> usize {
        let palette = main.palette(palette_id).unwrap();
        let property_id = palette.property_id_by_name(name).unwrap();
        let value_pool_id = palette.property(property_id).unwrap().value_pool_id;

        main.value_pool(value_pool_id).unwrap().len()
    }

    fn write(property: &str, expression: &str) -> PropertyWrite {
        PropertyWrite {
            property: property.to_owned(),
            expression: expression.to_owned(),
        }
    }

    /// Whether `error` rose from the write to `property` over `palette_id`.
    fn is_write_error(error: &Error, palette_id: U32Id<BVoxPalette>, property: &str) -> bool {
        let Error::InPalette {
            palette_id: error_palette_id,
            error,
        } = error
        else {
            return false;
        };

        *error_palette_id == palette_id
            && matches!(
                error.as_ref(),
                Error::PaletteEdit {
                    element: PaletteEditElement::Write { property: written },
                    ..
                } if written == property
            )
    }

    #[test]
    fn a_swatch_array_lands_one_entry_per_material() {
        let mut main = VoxMain::default();
        let palette_id = retain_tagged(&mut main, &["rust", "steel"], &[0.2, 0.4]);

        edit_palettes(
            &mut main,
            &[palette_id],
            "rust = tag == \"rust\";",
            &[write(ROUGHNESS, "mix(roughness, 0.9, rust)")],
        )
        .unwrap();

        assert_eq!(
            drawn(&main, palette_id, ROUGHNESS, VoxValuePool::float_values),
            [0.9, 0.4]
        );
        // 0.2 is released because no material draws it anymore.
        assert_eq!(value_count(&main, palette_id, ROUGHNESS), 2);
        assert_eq!(main.validate(), Ok(()));
    }

    #[test]
    fn a_plain_value_lands_on_every_material_reusing_an_equal_value() {
        let mut main = VoxMain::default();
        let palette_id = retain_tagged(&mut main, &["rust", "steel"], &[0.2, 0.4]);

        edit_palettes(&mut main, &[palette_id], "", &[write(ROUGHNESS, "0.4")]).unwrap();

        assert_eq!(
            drawn(&main, palette_id, ROUGHNESS, VoxValuePool::float_values),
            [0.4, 0.4]
        );
        assert_eq!(value_count(&main, palette_id, ROUGHNESS), 1);
    }

    #[test]
    fn a_missing_property_is_added_on_a_new_value_pool() {
        let mut main = VoxMain::default();
        let palette_id = retain_tagged(&mut main, &["rust", "steel"], &[0.2, 0.4]);
        let value_pool_count = main.value_pool_count();

        edit_palettes(
            &mut main,
            &[palette_id],
            "",
            &[write("wear", "mix(0u32, 1, tag == \"rust\")")],
        )
        .unwrap();

        assert_eq!(
            drawn(&main, palette_id, "wear", VoxValuePool::int_values),
            [1, 0]
        );
        assert_eq!(main.value_pool_count(), value_pool_count + 1);
        assert_eq!(main.validate(), Ok(()));
    }

    #[test]
    fn a_materialless_palette_gains_the_property_on_an_empty_value_pool() {
        let mut main = VoxMain::default();
        let palette_id = main.retain_palette(VoxPalette::default()).unwrap();

        edit_palettes(&mut main, &[palette_id], "", &[write("wear", "1.0")]).unwrap();

        assert_eq!(value_count(&main, palette_id, "wear"), 0);
        assert_eq!(main.validate(), Ok(()));
    }

    #[test]
    fn a_palette_sharing_the_value_pool_keeps_its_cells() {
        let mut main = VoxMain::default();
        let value_pool_id = main.retain_value_pool(VoxValuePool::float(vec![0.2, 0.4]).unwrap());

        let mut retain_shared = |value_indices: &[u32]| {
            let mut palette = VoxPalette::default();

            palette
                .retain_property(ROUGHNESS.to_owned(), value_pool_id)
                .unwrap();

            for &index in value_indices {
                palette
                    .retain_material(vec![U32Id::from_u32(index)])
                    .unwrap();
            }

            main.retain_palette(palette).unwrap()
        };

        let edited_id = retain_shared(&[0, 1]);
        let kept_id = retain_shared(&[0]);

        edit_palettes(&mut main, &[edited_id], "", &[write(ROUGHNESS, "0.9")]).unwrap();

        assert_eq!(
            drawn(&main, edited_id, ROUGHNESS, VoxValuePool::float_values),
            [0.9, 0.9]
        );
        assert_eq!(
            drawn(&main, kept_id, ROUGHNESS, VoxValuePool::float_values),
            [0.2]
        );
        // 0.2 stays for the kept palette, 0.4 is released, and 0.9 is new.
        assert_eq!(main.value_pool(value_pool_id).unwrap().len(), 2);
        assert_eq!(main.validate(), Ok(()));
    }

    #[test]
    fn a_name_one_palette_lacks_errors_reporting_it_and_changes_nothing() {
        let mut main = VoxMain::default();
        let tagged_id = retain_tagged(&mut main, &["rust"], &[0.2]);
        let untagged_id = retain_palette(
            &mut main,
            vec![(ROUGHNESS, VoxValuePool::float(vec![0.4]).unwrap())],
            &[vec![0]],
        );

        let error = edit_palettes(
            &mut main,
            &[tagged_id, untagged_id],
            "",
            &[write(ROUGHNESS, "mix(roughness, 0.9, tag == \"rust\")")],
        )
        .unwrap_err();

        assert!(is_write_error(&error, untagged_id, ROUGHNESS), "{error}");
        assert!(
            error
                .to_string()
                .starts_with(&format!("palette {untagged_id}: the write to `roughness`")),
            "{error}"
        );
        assert_eq!(
            drawn(&main, tagged_id, ROUGHNESS, VoxValuePool::float_values),
            [0.2]
        );
    }

    #[test]
    fn default_covers_a_name_some_palettes_lack() {
        let mut main = VoxMain::default();
        let tagged_id = retain_tagged(&mut main, &["rust"], &[0.2]);
        let untagged_id = retain_palette(
            &mut main,
            vec![(ROUGHNESS, VoxValuePool::float(vec![0.4]).unwrap())],
            &[vec![0]],
        );

        edit_palettes(
            &mut main,
            &[tagged_id, untagged_id],
            "rust = default(tag, \"\") == \"rust\";",
            &[write(ROUGHNESS, "mix(roughness, 0.9, rust)")],
        )
        .unwrap();

        assert_eq!(
            drawn(&main, tagged_id, ROUGHNESS, VoxValuePool::float_values),
            [0.9]
        );
        assert_eq!(
            drawn(&main, untagged_id, ROUGHNESS, VoxValuePool::float_values),
            [0.4]
        );
    }

    #[test]
    fn a_json_property_stays_unbound() {
        let mut main = VoxMain::default();
        let palette_id = retain_palette(
            &mut main,
            vec![("meta", VoxValuePool::json(vec![VoxValue::Null]))],
            &[vec![0]],
        );

        let error =
            edit_palettes(&mut main, &[palette_id], "", &[write("wear", "meta")]).unwrap_err();

        assert!(is_write_error(&error, palette_id, "wear"), "{error}");
    }

    #[test]
    fn two_writes_to_one_property_error() {
        let mut main = VoxMain::default();
        let palette_id = retain_tagged(&mut main, &["rust"], &[0.2]);

        let error = edit_palettes(
            &mut main,
            &[palette_id],
            "",
            &[write(ROUGHNESS, "0.5"), write(ROUGHNESS, "0.6")],
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "the write to `roughness` repeats");
    }

    #[test]
    fn a_kind_other_than_the_value_pool_errors_changing_nothing() {
        let mut main = VoxMain::default();
        let palette_id = retain_tagged(&mut main, &["rust"], &[0.2]);

        let error =
            edit_palettes(&mut main, &[palette_id], "", &[write(ROUGHNESS, "1u32")]).unwrap_err();

        assert!(is_write_error(&error, palette_id, ROUGHNESS), "{error}");
        assert_eq!(
            drawn(&main, palette_id, ROUGHNESS, VoxValuePool::float_values),
            [0.2]
        );
    }

    #[test]
    fn a_vocabulary_value_outside_its_range_errors() {
        let mut main = VoxMain::default();
        let palette_id = retain_tagged(&mut main, &["rust"], &[0.2]);

        let error =
            edit_palettes(&mut main, &[palette_id], "", &[write(ROUGHNESS, "1.2")]).unwrap_err();

        let message = error.to_string();
        assert!(
            message.starts_with(&format!("palette {palette_id}: `roughness` is 1.2")),
            "{message}"
        );
        assert_eq!(
            drawn(&main, palette_id, ROUGHNESS, VoxValuePool::float_values),
            [0.2]
        );
    }

    #[test]
    fn an_unknown_palette_errors() {
        let mut main = VoxMain::default();
        let palette_id = retain_tagged(&mut main, &["rust"], &[0.2]);

        let released_id = main.retain_palette(VoxPalette::default()).unwrap();

        main.release_palette(released_id).unwrap();

        let error = edit_palettes(
            &mut main,
            &[palette_id, released_id],
            "",
            &[write(ROUGHNESS, "0.5")],
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("palette {released_id} is not one of this state's")
        );
    }
}
