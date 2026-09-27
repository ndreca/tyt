use crate::{Error, ObjectSelection, Result};
use branded_id::U32Id;
use voxcore::{BVoxObject, VoxExt, VoxMain};

/// The objects `selection` resolves to in `main`, in document order. A
/// document holding none is a usage error; `resolve` already rejects a
/// selector matching nothing.
pub(crate) fn select_mesh_objects<T: VoxExt>(
    main: &VoxMain<T>,
    selection: &ObjectSelection,
) -> Result<Vec<U32Id<BVoxObject>>> {
    let object_ids = selection.resolve(main)?;

    if object_ids.is_empty() {
        return Err(Error::usage("the document has no objects to mesh"));
    }

    Ok(object_ids)
}

#[cfg(test)]
mod tests {
    use crate::{ObjectSelection, commands::select_mesh_objects};
    use clap::Parser;
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject};

    /// A command carrying only the selectors.
    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        selection: ObjectSelection,
    }

    /// The selectors parsed from `args`.
    fn selection(args: &[&str]) -> ObjectSelection {
        let mut argv = vec!["cli"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).unwrap().selection
    }

    #[test]
    fn every_selected_object_is_meshed_and_an_empty_document_errors() {
        let mut main: VoxMain = VoxMain::default();
        let a = main
            .retain_object(VoxObject::new("a".to_owned(), TyVector3U32::ONE).unwrap())
            .unwrap();
        let b = main
            .retain_object(VoxObject::new("b".to_owned(), TyVector3U32::ONE).unwrap())
            .unwrap();

        assert_eq!(
            select_mesh_objects(&main, &selection(&[])).unwrap(),
            vec![a, b]
        );
        assert_eq!(
            select_mesh_objects(&main, &selection(&["--select-index", "1"])).unwrap(),
            vec![b]
        );

        let empty: VoxMain = VoxMain::default();
        assert!(select_mesh_objects(&empty, &selection(&[])).is_err());
    }
}
