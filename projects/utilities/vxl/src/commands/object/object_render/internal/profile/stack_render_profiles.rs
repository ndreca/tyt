use crate::{
    Error, ProfileSet, Result,
    commands::{RenderProfile, ViewEntry},
};
use std::collections::{BTreeMap, BTreeSet};

/// Stacks the profiles `names` into one profile to apply whole. Views merge
/// by name, element by element. The lights merge as one element, the rig.
/// A profile's `viewsFrom` and `lightsFrom` imports land depth-first ahead of
/// its views and rig. Each profile's views and rig land once, however many
/// members and imports bring them. Errors if a name repeats, an import
/// cycles, or two profiles set the same element.
///
/// # Arguments
/// * `origin` - where `names` came from, which errors report.
// Nothing reads this until the render command lands.
#[allow(dead_code)]
pub fn stack_render_profiles(
    profiles: &ProfileSet<RenderProfile>,
    origin: &str,
    names: &[String],
) -> Result<RenderProfile> {
    let mut stack = Stack::default();

    for (position, name) in (0..).zip(names) {
        if names[..position].contains(name) {
            return Err(Error::usage(format!("{origin} lists `{name}` twice")));
        }

        let member = profiles.get(origin, name)?;

        if let Some(width) = member.width {
            stack.claim(name, "width".to_owned())?;
            stack.profile.width = Some(width);
        }

        if let Some(height) = member.height {
            stack.claim(name, "height".to_owned())?;
            stack.profile.height = Some(height);
        }

        if let Some(background) = member.background {
            stack.claim(name, "background".to_owned())?;
            stack.profile.background = Some(background);
        }

        if let Some(occlusion) = member.occlusion {
            stack.claim(name, "occlusion".to_owned())?;
            stack.profile.occlusion = Some(occlusion);
        }

        if let Some(voxel_size) = member.voxel_size {
            stack.claim(name, "voxelSize".to_owned())?;
            stack.profile.voxel_size = Some(voxel_size);
        }

        stack.land_views(profiles, origin, name, &mut Vec::new())?;
        stack.land_lights(profiles, origin, name, &mut Vec::new())?;
    }

    Ok(stack.profile)
}

#[derive(Default)]
struct Stack {
    profile: RenderProfile,

    // Each set element and the profile that set it.
    claims: BTreeMap<String, String>,

    landed_views: BTreeSet<String>,

    landed_lights: BTreeSet<String>,
}

impl Stack {
    /// Lands the views of the profile `name`, which `origin` asks for, after
    /// its `viewsFrom` imports.
    fn land_views(
        &mut self,
        profiles: &ProfileSet<RenderProfile>,
        origin: &str,
        name: &str,
        visiting: &mut Vec<String>,
    ) -> Result<()> {
        check_cycle(visiting, name, "viewsFrom")?;

        if self.landed_views.contains(name) {
            return Ok(());
        }

        let profile = profiles.get(origin, name)?;

        visiting.push(name.to_owned());

        for import in &profile.views_from {
            self.land_views(
                profiles,
                &format!("the profile `{name}`'s viewsFrom"),
                import,
                visiting,
            )?;
        }

        visiting.pop();

        for (view_name, entry) in &profile.views {
            let entry_origin = format!("view `{view_name}`");

            if let Some(transform) = entry.transform {
                self.claim(name, format!("{entry_origin}'s transform"))?;
                self.view(view_name).transform = Some(transform);
            }

            if let Some(projection) = entry.projection {
                self.claim(name, format!("{entry_origin}'s projection"))?;
                self.view(view_name).projection = Some(projection);
            }

            if let Some(fov) = entry.fov {
                self.claim(name, format!("{entry_origin}'s fov"))?;
                self.view(view_name).fov = Some(fov);
            }

            if let Some(scale) = entry.scale {
                self.claim(name, format!("{entry_origin}'s scale"))?;
                self.view(view_name).scale = Some(scale);
            }

            if let Some(select) = &entry.select {
                self.claim(name, format!("{entry_origin}'s select"))?;
                self.view(view_name).select = Some(select.clone());
            }
        }

        self.landed_views.insert(name.to_owned());

        Ok(())
    }

    /// Lands the rig of the profile `name`, which `origin` asks for, after its
    /// `lightsFrom` imports.
    fn land_lights(
        &mut self,
        profiles: &ProfileSet<RenderProfile>,
        origin: &str,
        name: &str,
        visiting: &mut Vec<String>,
    ) -> Result<()> {
        check_cycle(visiting, name, "lightsFrom")?;

        if self.landed_lights.contains(name) {
            return Ok(());
        }

        let profile = profiles.get(origin, name)?;

        visiting.push(name.to_owned());

        for import in &profile.lights_from {
            self.land_lights(
                profiles,
                &format!("the profile `{name}`'s lightsFrom"),
                import,
                visiting,
            )?;
        }

        visiting.pop();

        if !profile.lights.is_empty() {
            self.claim(name, "lights".to_owned())?;
            self.profile.lights = profile.lights.clone();
        }

        self.landed_lights.insert(name.to_owned());

        Ok(())
    }

    fn view(&mut self, name: &str) -> &mut ViewEntry {
        self.profile.views.entry(name.to_owned()).or_default()
    }

    fn claim(&mut self, name: &str, element: String) -> Result<()> {
        if let Some(earlier) = self.claims.get(&element) {
            return Err(Error::usage(format!(
                "the profile `{name}` sets {element}, which the profile `{earlier}` sets already"
            )));
        }

        self.claims.insert(element, name.to_owned());

        Ok(())
    }
}

/// Errors when `name` already sits on the import chain `visiting`. `key` is
/// the import key the error reports.
fn check_cycle(visiting: &[String], name: &str, key: &str) -> Result<()> {
    let Some(start) = visiting.iter().position(|visited| visited == name) else {
        return Ok(());
    };

    let chain: Vec<_> = visiting[start..]
        .iter()
        .map(String::as_str)
        .chain([name])
        .map(|name| format!("`{name}`"))
        .collect();

    Err(Error::usage(format!(
        "the profile `{name}`'s {key} cycles: {}",
        chain.join(" imports ")
    )))
}

#[cfg(test)]
mod tests {
    use crate::{
        NamedCliValue, ProfileSet,
        commands::{
            ProjectionKind, RenderProfile, built_in_render_profiles, stack_render_profiles,
        },
    };
    use std::collections::BTreeMap;

    /// A set holding one profile per `(name, json)` pair of `entries`.
    fn profiles(entries: &[(&str, &str)]) -> ProfileSet<RenderProfile> {
        let profiles: BTreeMap<String, RenderProfile> = entries
            .iter()
            .map(|(name, json)| ((*name).to_owned(), serde_json::from_str(json).unwrap()))
            .collect();

        ProfileSet::from_profiles(profiles)
    }

    fn names(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn views_merge_by_name_and_a_rig_joins_a_view_set() {
        let profiles = profiles(&[
            (
                "views",
                r#"{
                    "width": 256,
                    "views": {
                        "hero": { "transform": { "kind": "orbit", "azimuth": 45, "elevation": 30 } },
                        "plan": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 90 } }
                    }
                }"#,
            ),
            (
                "flat-plan",
                r#"{
                    "height": 128,
                    "views": { "plan": { "projection": "orthographic" } }
                }"#,
            ),
            (
                "rig",
                r#"{ "lights": [{ "kind": "hemisphere", "strength": 1 }] }"#,
            ),
        ]);

        let stack = stack_render_profiles(
            &profiles,
            "--profile",
            &names(&["views", "flat-plan", "rig"]),
        )
        .unwrap();

        assert_eq!(stack.width.map(u32::from), Some(256));
        assert_eq!(stack.height.map(u32::from), Some(128));
        assert_eq!(stack.views.keys().collect::<Vec<_>>(), ["hero", "plan"]);
        assert!(stack.views["plan"].transform.is_some());
        assert_eq!(
            stack.views["plan"].projection,
            Some(NamedCliValue(ProjectionKind::Orthographic))
        );
        assert_eq!(stack.lights.len(), 1);
    }

    #[test]
    fn an_element_two_members_set_errors_naming_both() {
        let profiles = profiles(&[
            ("a", r#"{ "views": { "hero": { "fov": 40 } } }"#),
            ("b", r#"{ "views": { "hero": { "fov": 50 } } }"#),
            ("wide", r#"{ "width": 1 }"#),
            ("wider", r#"{ "width": 2 }"#),
        ]);

        let error = stack_render_profiles(&profiles, "--profile", &names(&["a", "b"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(
                "the profile `b` sets view `hero`'s fov, which the profile `a` sets already"
            ),
            "{error}"
        );

        let error = stack_render_profiles(&profiles, "--profile", &names(&["wide", "wider"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("`wider` sets width"), "{error}");
    }

    #[test]
    fn two_rigs_error_and_a_member_listed_twice_errors() {
        let profiles = ProfileSet::layered(built_in_render_profiles(), []);

        let error = stack_render_profiles(&profiles, "--profile", &names(&["studio", "flat"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("`flat` sets lights"), "{error}");

        let error = stack_render_profiles(&profiles, "--profile", &names(&["hero", "hero"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("--profile lists `hero` twice"), "{error}");

        let stack =
            stack_render_profiles(&profiles, "--profile", &names(&["turnaround", "studio"]))
                .unwrap();
        assert_eq!(
            stack.views.keys().collect::<Vec<_>>(),
            ["back", "front", "hero", "left", "right"]
        );
        assert_eq!(stack.lights.len(), 2);
    }

    #[test]
    fn views_from_imports_views_alone_ahead_of_the_profile_s_own() {
        let profiles = profiles(&[
            (
                "base",
                r#"{
                    "width": 256,
                    "views": { "hero": { "transform": { "kind": "orbit", "azimuth": 45, "elevation": 30 } } },
                    "lights": [{ "kind": "hemisphere", "strength": 1 }]
                }"#,
            ),
            (
                "wide-hero",
                r#"{ "viewsFrom": ["base"], "views": { "hero": { "fov": 60 } } }"#,
            ),
            (
                "clash",
                r#"{ "viewsFrom": ["base"], "views": { "hero": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 } } } }"#,
            ),
        ]);

        let stack = stack_render_profiles(&profiles, "--profile", &names(&["wide-hero"])).unwrap();
        assert!(stack.views["hero"].transform.is_some());
        assert!(stack.views["hero"].fov.is_some());
        assert_eq!(stack.width, None);
        assert!(stack.lights.is_empty());

        let error = stack_render_profiles(&profiles, "--profile", &names(&["clash"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(
                "the profile `clash` sets view `hero`'s transform, which the profile `base` sets already"
            ),
            "{error}"
        );
    }

    #[test]
    fn lights_from_imports_one_rig() {
        let mut profiles = built_in_render_profiles();
        for (name, json) in [
            (
                "lit",
                r#"{ "viewsFrom": ["top"], "lightsFrom": ["studio"] }"#,
            ),
            ("two-rigs", r#"{ "lightsFrom": ["studio", "flat"] }"#),
            (
                "own-rig",
                r#"{ "lightsFrom": ["studio"], "lights": [{ "kind": "hemisphere" }] }"#,
            ),
        ] {
            profiles.insert(name.to_owned(), serde_json::from_str(json).unwrap());
        }
        let profiles = ProfileSet::from_profiles(profiles);

        let stack = stack_render_profiles(&profiles, "--profile", &names(&["lit"])).unwrap();
        assert_eq!(stack.views.keys().collect::<Vec<_>>(), ["top"]);
        assert_eq!(stack.lights.len(), 2);

        let error = stack_render_profiles(&profiles, "--profile", &names(&["two-rigs"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("`flat` sets lights"), "{error}");

        let error = stack_render_profiles(&profiles, "--profile", &names(&["own-rig"]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("`own-rig` sets lights"), "{error}");
    }

    #[test]
    fn a_profile_s_views_land_once_however_many_bring_them() {
        let profiles = profiles(&[
            (
                "d",
                r#"{ "views": { "d": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 } } } }"#,
            ),
            ("b", r#"{ "viewsFrom": ["d"] }"#),
            ("c", r#"{ "viewsFrom": ["d"] }"#),
            ("a", r#"{ "viewsFrom": ["b", "c"] }"#),
        ]);

        let stack = stack_render_profiles(&profiles, "--profile", &names(&["a", "d"])).unwrap();
        assert_eq!(stack.views.keys().collect::<Vec<_>>(), ["d"]);
    }

    #[test]
    fn an_import_cycle_and_an_unknown_import_error() {
        let profiles = profiles(&[
            ("a", r#"{ "viewsFrom": ["b"] }"#),
            ("b", r#"{ "viewsFrom": ["a"] }"#),
            ("lost", r#"{ "lightsFrom": ["nowhere"] }"#),
        ]);

        let error = stack_render_profiles(&profiles, "--profile", &names(&["a"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `a`'s viewsFrom cycles: `a` imports `b` imports `a`"),
            "{error}"
        );

        let error = stack_render_profiles(&profiles, "--profile", &names(&["lost"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("the profile `lost`'s lightsFrom asks for the profile `nowhere`"),
            "{error}"
        );
    }
}
