use crate::{
    SdfjMaterial, SdfjNode, SdfjObject, SdfjPattern, SdfjShades, SdfjShape2d, SdfjShape3d, SdfjStep,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The root of an SDF Json document. Each table holds one kind of value a
/// model can share, and entries reference each other by index. Every key is
/// required.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjFile {
    /// The format version. A reader takes only
    /// [`SDFJ_VERSION`](crate::SDFJ_VERSION).
    pub version: u32,

    /// The 3D shapes.
    pub shapes3d: Vec<SdfjShape3d>,

    /// The 2D shapes.
    pub shapes2d: Vec<SdfjShape2d>,

    /// The materials.
    pub materials: Vec<SdfjMaterial>,

    /// The `shades` calls.
    pub shades: Vec<SdfjShades>,

    /// The patterns.
    pub patterns: Vec<SdfjPattern>,

    /// The steps.
    pub steps: Vec<SdfjStep>,

    /// The parts' objects.
    pub objects: Vec<SdfjObject>,

    /// The parts' nodes.
    pub nodes: Vec<SdfjNode>,

    /// Indices into [`nodes`](SdfjFile::nodes): the nodes at the top of the
    /// hierarchy.
    pub root_nodes: Vec<usize>,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{
        SdfjFile, SdfjIntValue, SdfjMaterial, SdfjPropertyValue, SdfjStep, SdfjTaggedValue,
        SdfjValue,
    };
    use serde_json::{Value, json};

    /// The legs and seat of a chair.
    fn chair() -> Value {
        json!({
            "version": 1,
            "shapes3d": [
                {
                    "kind": "lathe",
                    "points": [[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]],
                },
                { "kind": "translate", "shape": 0, "offset": [0.2, 0, 0.2] },
                { "kind": "mirror", "shape": 1, "axes": "xz" },
                { "kind": "box", "min": [-0.25, 0.425, -0.25], "max": [0.25, 0.475, 0.25] },
            ],
            "shapes2d": [],
            "materials": [
                { "kind": "material", "properties": { "baseColor": "#8A5A2B", "roughness": 0.8 } },
                { "kind": "shade", "shades": 0, "index": 0 },
                { "kind": "shade", "shades": 0, "index": 1 },
                { "kind": "shade", "shades": 0, "index": 2 },
            ],
            "shades": [{ "base": 0, "count": 3 }],
            "patterns": [
                { "kind": "grain", "materials": [1, 2, 3], "axis": "y", "seed": 1 },
            ],
            "steps": [
                { "kind": "add", "name": "legs", "shape": 2, "pattern": 0 },
                { "kind": "add", "name": "seat", "shape": 3, "material": 0 },
            ],
            "objects": [{ "name": "chair", "steps": [0, 1] }],
            "nodes": [{ "name": "chair", "pivot": [0, 0, 0], "childObjects": [0], "childNodes": [] }],
            "rootNodes": [0],
        })
    }

    /// The chair with `edit` applied to its JSON.
    fn chair_with(edit: impl FnOnce(&mut Value)) -> Value {
        let mut chair = chair();

        edit(&mut chair);

        chair
    }

    /// The first material's properties after reading `properties`.
    fn properties(properties: Value) -> Vec<SdfjPropertyValue> {
        let chair = chair_with(|chair| chair["materials"][0]["properties"] = properties);

        let file: SdfjFile = serde_json::from_value(chair).unwrap();

        let SdfjMaterial::Material { properties } = file.materials[0].clone() else {
            panic!("the first material holds properties");
        };

        properties
            .into_entries()
            .into_iter()
            .map(|entry| entry.value)
            .collect()
    }

    #[test]
    fn a_document_round_trips() {
        let file: SdfjFile = serde_json::from_value(chair()).unwrap();

        let text = serde_json::to_string(&file).unwrap();

        assert_eq!(serde_json::from_str::<SdfjFile>(&text).unwrap(), file);

        assert_eq!(
            file.steps[0],
            SdfjStep::Add {
                name: "legs".to_owned(),
                shape: 2,
                material: None,
                pattern: Some(0),
            }
        );
    }

    #[test]
    fn a_key_left_out_of_an_entry_stays_out_on_write() {
        let file: SdfjFile = serde_json::from_value(chair()).unwrap();

        let value = serde_json::to_value(&file).unwrap();

        assert_eq!(value["steps"][0].get("material"), None);

        assert_eq!(value["shades"][0].get("spread"), None);

        assert_eq!(value["nodes"][0].get("offset"), None);
    }

    #[test]
    fn unknown_and_missing_keys_fail_to_read() {
        let documents = [
            chair_with(|chair| chair["extra"] = json!([])),
            chair_with(|chair| {
                chair.as_object_mut().unwrap().remove("shapes2d");
            }),
            chair_with(|chair| chair["shapes3d"][3]["center"] = json!([0, 0, 0])),
            chair_with(|chair| chair["nodes"][0]["extra"] = json!(1)),
            chair_with(|chair| chair["shades"][0]["extra"] = json!(1)),
        ];

        for document in documents {
            assert!(serde_json::from_value::<SdfjFile>(document).is_err());
        }
    }

    #[test]
    fn an_unknown_kind_fails_to_read() {
        let chair = chair_with(|chair| chair["shapes3d"][3]["kind"] = json!("cube"));

        assert!(serde_json::from_value::<SdfjFile>(chair).is_err());
    }

    #[test]
    fn an_explicit_null_fails_to_read() {
        let documents = [
            chair_with(|chair| chair["shapes3d"][3]["round"] = Value::Null),
            chair_with(|chair| chair["nodes"][0]["pivot"] = Value::Null),
            chair_with(|chair| chair["steps"][0]["material"] = Value::Null),
        ];

        for document in documents {
            assert!(serde_json::from_value::<SdfjFile>(document).is_err());
        }
    }

    #[test]
    fn an_index_reads_only_as_a_whole_number() {
        let documents = [
            chair_with(|chair| chair["steps"][0]["shape"] = json!(2.0)),
            chair_with(|chair| chair["rootNodes"] = json!([-1])),
        ];

        for document in documents {
            assert!(serde_json::from_value::<SdfjFile>(document).is_err());
        }
    }

    #[test]
    fn each_property_form_reads_as_its_value() {
        let values = properties(json!({
            "bool": true,
            "int": { "kind": "int", "value": 3 },
            "ints": { "kind": "int", "value": [1, 2] },
            "json": { "kind": "json", "value": { "k": null } },
            "number": 0.5,
            "numbers": [0.5, 1],
            "text": "#FFFFFF",
        }));

        assert_eq!(
            values,
            [
                SdfjPropertyValue::Bool(true),
                SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(SdfjIntValue::Number(3.0))),
                SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(SdfjIntValue::NumberArray(vec![
                    1.0, 2.0
                ]))),
                SdfjPropertyValue::Tagged(SdfjTaggedValue::Json(
                    serde_json::from_value::<SdfjValue>(json!({ "k": null })).unwrap()
                )),
                SdfjPropertyValue::Number(0.5),
                SdfjPropertyValue::NumberArray(vec![0.5, 1.0]),
                SdfjPropertyValue::Text("#FFFFFF".to_owned()),
            ]
        );
    }

    #[test]
    fn a_malformed_tagged_value_fails_to_read() {
        let documents = [
            json!({ "p": { "kind": "float", "value": 3 } }),
            json!({ "p": { "kind": "int", "value": "3" } }),
            json!({ "p": { "kind": "int", "value": 3, "extra": 1 } }),
            json!({ "p": null }),
        ];

        for document in documents {
            let chair = chair_with(|chair| chair["materials"][0]["properties"] = document);

            assert!(serde_json::from_value::<SdfjFile>(chair).is_err());
        }
    }
}
