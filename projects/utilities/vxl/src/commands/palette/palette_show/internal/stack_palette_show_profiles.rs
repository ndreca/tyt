use crate::{Error, ProfileSet, Result, commands::PaletteShowProfile};

/// The profiles `names`, which `origin` lists, stacked into one profile to
/// apply whole. A layout two members both set errors. The stack carries no
/// selectors because each member lands its selectors and imports by name.
pub(crate) fn stack_palette_show_profiles(
    profiles: &ProfileSet<PaletteShowProfile>,
    origin: &str,
    names: &[String],
) -> Result<PaletteShowProfile> {
    let mut stack = PaletteShowProfile::default();
    let mut layout_claim: Option<&str> = None;

    for (position, name) in (0..).zip(names) {
        if names[..position].contains(name) {
            return Err(Error::usage(format!("{origin} lists `{name}` twice")));
        }

        let member = profiles.get(origin, name)?;

        if let Some(layout) = member.layout {
            if let Some(earlier) = layout_claim {
                return Err(Error::usage(format!(
                    "the profile `{name}` sets layout, which the profile `{earlier}` sets already"
                )));
            }

            layout_claim = Some(name);
            stack.layout = Some(layout);
        }
    }

    Ok(stack)
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet,
        commands::{PaletteShowLayoutEntry, PaletteShowProfile, stack_palette_show_profiles},
    };
    use std::collections::BTreeMap;
    use voxsmith::operations::palette::{PaletteShowLayout, PaletteShowTableShape};

    /// A set holding the profiles `entries` defines as json.
    fn profiles(entries: &[(&str, &str)]) -> ProfileSet<PaletteShowProfile> {
        let profiles: BTreeMap<String, PaletteShowProfile> = entries
            .iter()
            .map(|(name, json)| ((*name).to_owned(), serde_json::from_str(json).unwrap()))
            .collect();

        ProfileSet::from_profiles(profiles)
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn the_layout_stacks_and_the_selectors_stay_behind() {
        let profiles = profiles(&[
            ("pbr", r#"{ "properties": [{ "property": "baseColor" }] }"#),
            (
                "table",
                r#"{ "layout": { "kind": "md-tables", "tableShape": "flat" } }"#,
            ),
        ]);

        let stack =
            stack_palette_show_profiles(&profiles, "--profile", &names(&["pbr", "table"])).unwrap();

        assert!(stack.properties.is_empty());
        assert_eq!(
            stack.layout,
            Some(PaletteShowLayoutEntry {
                table_shape: Some(PaletteShowTableShape::Flat),
                ..PaletteShowLayoutEntry::from(PaletteShowLayout::MdTables)
            })
        );
    }

    #[test]
    fn a_layout_two_members_set_errors_naming_both() {
        let profiles = profiles(&[
            ("rows", r#"{ "layout": "text-rows" }"#),
            ("table", r#"{ "layout": "md-tables" }"#),
        ]);

        let error = stack_palette_show_profiles(&profiles, "--profile", &names(&["rows", "table"]))
            .unwrap_err()
            .to_string();
        assert!(
            error
                .contains("the profile `table` sets layout, which the profile `rows` sets already"),
            "{error}"
        );
    }

    #[test]
    fn a_member_listed_twice_errors() {
        let profiles = profiles(&[("rows", r#"{ "layout": "text-rows" }"#)]);

        let error = stack_palette_show_profiles(&profiles, "--profile", &names(&["rows", "rows"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("--profile lists `rows` twice"), "{error}");
    }
}
