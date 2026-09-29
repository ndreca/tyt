use crate::CliValue;
use voxsmith::{
    operations::palette::{
        PaletteRef, PaletteShowPresentation, PaletteShowReading, PropertyRef, PropertySelector,
    },
    utilities::VectorComponent,
};

/// Parses one `--property <palette> <property> <presentation> <reading>`
/// selector for `palette show` from its four fields. `*` matches every palette
/// or property.
pub fn parse_property_selector(
    palette: &str,
    property: &str,
    presentation: &str,
    reading: &str,
) -> Result<PropertySelector, String> {
    Ok(PropertySelector {
        palette: parse_palette_ref(palette)?,
        property: parse_property_ref(property)?,
        presentation: PaletteShowPresentation::parse(presentation)?,
        reading: PaletteShowReading::parse(reading)?,
    })
}

/// Parses the palette field of a `--property` selector: `*` for every
/// palette, else a non-negative index.
fn parse_palette_ref(text: &str) -> Result<PaletteRef, String> {
    if text == "*" {
        return Ok(PaletteRef::All);
    }

    text.parse::<usize>()
        .map(PaletteRef::Index)
        .map_err(|_| format!("`{text}` is not a palette index or `*`"))
}

/// Parses the property field of a `--property` selector: `*` for every
/// property, else a key with an optional trailing `.r`/`.g`/`.b`/`.a` or
/// `.x`/`.y`/`.z`/`.w` component. A longer dotted suffix stays part of the
/// key.
fn parse_property_ref(text: &str) -> Result<PropertyRef, String> {
    if text == "*" {
        return Ok(PropertyRef::All);
    }

    let (key, component) = match text.rsplit_once('.') {
        Some((head, tail)) if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) => {
            (head, Some(VectorComponent::parse(tail)?))
        }
        _ => (text, None),
    };

    if key.is_empty() {
        return Err(format!("`{text}` names no property"));
    }

    Ok(PropertyRef::Key {
        key: key.to_string(),
        component,
    })
}

#[cfg(test)]
mod tests {
    use crate::commands::{
        palette::palette_show::internal::parse_property_selector::{
            parse_palette_ref, parse_property_ref,
        },
        parse_property_selector,
    };
    use voxsmith::{
        operations::palette::{
            PaletteRef, PaletteShowPresentation, PaletteShowReading, PropertyRef,
        },
        utilities::VectorComponent,
    };

    #[test]
    fn parses_a_full_selector() {
        let selector = parse_property_selector("0", "rgba.a", "value", "srgb-hex").unwrap();

        assert_eq!(selector.palette, PaletteRef::Index(0));
        assert_eq!(
            selector.property,
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: Some(VectorComponent::A),
            }
        );
        assert_eq!(selector.presentation, PaletteShowPresentation::Value);
        assert_eq!(selector.reading, PaletteShowReading::SrgbHex);
    }

    #[test]
    fn parses_stars() {
        let selector = parse_property_selector("*", "*", "swatch", "auto").unwrap();

        assert_eq!(selector.palette, PaletteRef::All);
        assert_eq!(selector.property, PropertyRef::All);
    }

    #[test]
    fn rejects_an_unknown_presentation() {
        assert!(parse_property_selector("0", "rgba", "rainbow", "auto").is_err());
    }

    #[test]
    fn rejects_an_unknown_reading() {
        assert!(parse_property_selector("0", "rgba", "value", "rainbow").is_err());
    }

    #[test]
    fn parses_a_star_and_an_index() {
        assert_eq!(parse_palette_ref("*").unwrap(), PaletteRef::All);
        assert_eq!(parse_palette_ref("0").unwrap(), PaletteRef::Index(0));
        assert_eq!(parse_palette_ref("12").unwrap(), PaletteRef::Index(12));
    }

    #[test]
    fn rejects_a_non_index() {
        assert!(parse_palette_ref("a").is_err());
        assert!(parse_palette_ref("-1").is_err());
        assert!(parse_palette_ref("").is_err());
    }

    #[test]
    fn parses_a_star() {
        assert_eq!(parse_property_ref("*").unwrap(), PropertyRef::All);
    }

    #[test]
    fn parses_a_bare_key() {
        assert_eq!(
            parse_property_ref("rgba").unwrap(),
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn parses_a_trailing_component_from_either_alias_set() {
        assert_eq!(
            parse_property_ref("rgba.a").unwrap(),
            PropertyRef::Key {
                key: "rgba".to_string(),
                component: Some(VectorComponent::A),
            }
        );
        assert_eq!(
            parse_property_ref("normal.y").unwrap(),
            PropertyRef::Key {
                key: "normal".to_string(),
                component: Some(VectorComponent::Y),
            }
        );
    }

    #[test]
    fn keeps_a_dotted_key_without_a_component() {
        assert_eq!(
            parse_property_ref("my.attr").unwrap(),
            PropertyRef::Key {
                key: "my.attr".to_string(),
                component: None,
            }
        );
    }

    #[test]
    fn rejects_an_unknown_component() {
        assert!(parse_property_ref("rgba.q").is_err());
    }

    #[test]
    fn rejects_an_empty_key() {
        assert!(parse_property_ref("").is_err());
        assert!(parse_property_ref(".a").is_err());
    }
}
