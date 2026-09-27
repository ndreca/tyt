use crate::{
    Result,
    commands::{object_stems, split_output},
};
use branded_id::U32Id;
use std::path::{Path, PathBuf};
use voxcore::{BVoxObject, VoxExt, VoxMain};

/// One output of a meshing run and the objects it holds, each beside the stem
/// its files fill `{file-stem}` with.
#[derive(Debug, PartialEq)]
pub(crate) struct MeshRun {
    /// The mesh the run writes.
    pub output: PathBuf,

    /// The objects the mesh holds, in order, each with its file stem.
    pub targets: Vec<(U32Id<BVoxObject>, String)>,
}

impl MeshRun {
    /// The runs meshing `object_ids` of `main` to `output` under `base_stem`:
    /// one run holding every object, or with `split` one run per object beside
    /// `output` under the object's name. A run over several objects, or a
    /// split run, joins the object's name to the stem with a hyphen so the
    /// objects' files stay apart.
    pub(crate) fn plan<T: VoxExt>(
        main: &VoxMain<T>,
        output: &Path,
        base_stem: &str,
        object_ids: &[U32Id<BVoxObject>],
        split: bool,
    ) -> Result<Vec<MeshRun>> {
        if !split && object_ids.len() == 1 {
            return Ok(vec![MeshRun {
                output: output.to_path_buf(),
                targets: vec![(object_ids[0], base_stem.to_owned())],
            }]);
        }

        let stems = object_stems(main, object_ids)?;

        let targets = object_ids
            .iter()
            .zip(&stems)
            .map(|(&object_id, stem)| (object_id, format!("{base_stem}-{stem}")));

        if split {
            return Ok(targets
                .zip(&stems)
                .map(|(target, stem)| MeshRun {
                    output: split_output(output, stem),
                    targets: vec![target],
                })
                .collect());
        }

        Ok(vec![MeshRun {
            output: output.to_path_buf(),
            targets: targets.collect(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::MeshRun;
    use branded_id::U32Id;
    use std::path::{Path, PathBuf};
    use ty_math::TyVector3U32;
    use voxcore::{BVoxObject, VoxMain, VoxObject};

    /// A main holding `turret` and `base`, and their ids.
    fn main() -> (VoxMain, [U32Id<BVoxObject>; 2]) {
        let mut main: VoxMain = VoxMain::default();
        let ids = ["turret", "base"].map(|name| {
            main.retain_object(VoxObject::new(name.to_owned(), TyVector3U32::ONE).unwrap())
                .unwrap()
        });
        (main, ids)
    }

    #[test]
    fn one_object_keeps_the_output_and_the_stem() {
        let (main, [turret, _]) = main();

        let runs = MeshRun::plan(&main, Path::new("scene.glb"), "scene", &[turret], false).unwrap();

        assert_eq!(
            runs,
            vec![MeshRun {
                output: PathBuf::from("scene.glb"),
                targets: vec![(turret, "scene".to_owned())],
            }]
        );
    }

    #[test]
    fn several_objects_share_the_output_under_qualified_stems() {
        let (main, [turret, base]) = main();

        let runs = MeshRun::plan(
            &main,
            Path::new("scene.glb"),
            "scene",
            &[turret, base],
            false,
        )
        .unwrap();

        assert_eq!(
            runs,
            vec![MeshRun {
                output: PathBuf::from("scene.glb"),
                targets: vec![
                    (turret, "scene-turret".to_owned()),
                    (base, "scene-base".to_owned()),
                ],
            }]
        );
    }

    #[test]
    fn a_split_run_writes_one_output_per_object_even_for_one() {
        let (main, [turret, base]) = main();

        let runs = MeshRun::plan(
            &main,
            Path::new("out/scene.glb"),
            "custom",
            &[turret, base],
            true,
        )
        .unwrap();

        assert_eq!(
            runs,
            vec![
                MeshRun {
                    output: PathBuf::from("out/scene-turret.glb"),
                    targets: vec![(turret, "custom-turret".to_owned())],
                },
                MeshRun {
                    output: PathBuf::from("out/scene-base.glb"),
                    targets: vec![(base, "custom-base".to_owned())],
                },
            ]
        );

        let runs = MeshRun::plan(&main, Path::new("scene.glb"), "scene", &[base], true).unwrap();
        assert_eq!(runs[0].output, PathBuf::from("scene-base.glb"));
        assert_eq!(runs[0].targets, vec![(base, "scene-base".to_owned())]);
    }
}
