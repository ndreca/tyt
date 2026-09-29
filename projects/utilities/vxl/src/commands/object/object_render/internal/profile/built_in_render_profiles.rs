use crate::commands::RenderProfile;
use jsonc_parser::{ParseOptions, parse_to_serde_value};
use std::collections::BTreeMap;

/// The built-in render profiles, parsed from the jsonc map the binary
/// embeds.
pub fn built_in_render_profiles() -> BTreeMap<String, RenderProfile> {
    let options = ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    };

    parse_to_serde_value(include_str!("built_in_render_profiles.jsonc"), &options)
        .expect("the embedded profiles are well-formed jsonc following the schema")
}

#[cfg(test)]
mod tests {
    use crate::{
        Profile,
        commands::{LightEntry, PoseTransformEntry, built_in_render_profiles},
    };

    #[test]
    fn the_ten_built_ins_load_as_view_sets_and_light_rigs() {
        let profiles = built_in_render_profiles();

        assert_eq!(
            profiles.keys().collect::<Vec<_>>(),
            [
                "back",
                "bottom",
                "flat",
                "front",
                "hero",
                "left",
                "right",
                "studio",
                "top",
                "turnaround"
            ]
        );

        for name in ["hero", "front", "back", "left", "right", "top", "bottom"] {
            let profile = &profiles[name];
            assert!(profile.lights.is_empty(), "{name}");
            assert_eq!(profile.views.keys().collect::<Vec<_>>(), [name]);
            assert!(matches!(
                profile.views[name].transform,
                Some(PoseTransformEntry::Orbit { distance: None, .. })
            ));
        }

        assert_eq!(
            profiles["turnaround"].views_from,
            ["hero", "front", "right", "back", "left"]
        );
        assert!(profiles["turnaround"].views.is_empty());

        for name in ["studio", "flat"] {
            assert!(profiles[name].views.is_empty(), "{name}");
        }
        assert_eq!(profiles["studio"].lights.len(), 2);
        assert!(matches!(
            profiles["studio"].lights[0],
            LightEntry::Directional { shadow: None, .. }
        ));
        assert!(matches!(
            profiles["flat"].lights[0],
            LightEntry::Directional {
                shadow: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn every_built_in_carries_a_description() {
        for (name, profile) in built_in_render_profiles() {
            assert!(profile.description().is_some(), "{name}");
        }
    }
}
