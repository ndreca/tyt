use crate::{Error, Result};
use branded_id::U32Id;
use voxcore::{BVoxObject, VoxExt, VoxMain};

/// The stem component each object of `object_ids` lends to its files in a run
/// that tells the objects' files apart: the object's name. An empty name, a
/// path separator, or a name two of the objects share is a usage error.
pub(crate) fn object_stems<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
) -> Result<Vec<String>> {
    let mut stems: Vec<String> = Vec::with_capacity(object_ids.len());

    for &object_id in object_ids {
        let name = main
            .object(object_id)
            .expect("the selection resolved an id from the main's objects")
            .name();

        let index = object_id.to_u32();

        if name.is_empty() {
            return Err(Error::usage(format!(
                "object {index} has no name, and a run over several objects or `--split-files` \
                 names each object's files by it; name the object or select one"
            )));
        }

        if name.contains('/') || name.contains('\\') {
            return Err(Error::usage(format!(
                "object {index} is named `{name}`, which names its files, so it cannot contain \
                 a path separator"
            )));
        }

        if let Some(earlier) = stems.iter().position(|stem| stem == name) {
            return Err(Error::usage(format!(
                "objects {} and {index} share the name `{name}`, which names their files; \
                 select one or rename them",
                object_ids[earlier].to_u32()
            )));
        }

        stems.push(name.to_owned());
    }

    Ok(stems)
}

#[cfg(test)]
mod tests {
    use crate::commands::object_stems;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject};

    /// A main holding one object per name, and their ids.
    fn main_of(names: &[&str]) -> VoxMain {
        let mut main: VoxMain = VoxMain::default();
        for name in names {
            main.retain_object(VoxObject::new(name.to_string(), TyVector3U32::ONE).unwrap())
                .unwrap();
        }
        main
    }

    /// The stems of every object of the main named by `names`, or the error.
    fn stems_of(names: &[&str]) -> Result<Vec<String>, String> {
        let main = main_of(names);
        let object_ids: Vec<_> = main
            .iter_objects()
            .map(|(object_id, _)| object_id)
            .collect();
        object_stems(&main, &object_ids).map_err(|error| error.to_string())
    }

    #[test]
    fn the_stems_are_the_names() {
        assert_eq!(stems_of(&["turret", "base"]).unwrap(), ["turret", "base"]);
    }

    #[test]
    fn an_empty_name_a_path_or_a_shared_name_errors() {
        assert!(
            stems_of(&["turret", ""])
                .unwrap_err()
                .contains("object 1 has no name")
        );
        assert!(
            stems_of(&["a/b"])
                .unwrap_err()
                .contains("cannot contain a path separator")
        );
        assert!(
            stems_of(&["turret", "base", "turret"])
                .unwrap_err()
                .contains("objects 0 and 2 share the name `turret`")
        );
    }
}
