use crate::{Error, Result, parse_index_range};
use branded_id::U32Id;
use clap::Args;
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxExt, VoxMain};
use voxsmith::utilities::{IndexRange, select_nodes, select_objects};

/// The `--select` / `--select-index` selectors of an edit command, which
/// requires at least one.
#[derive(Clone, Debug, Args)]
#[group(required = true, multiple = true)]
pub struct RequiredSelection {
    /// Choose by hierarchy-path glob. An object command takes every object at
    /// or under a matched path, and a node command takes the matched node
    /// alone. Repeatable; unions with `--select-index`.
    #[arg(value_name = "select", long)]
    select: Vec<String>,

    /// Choose by index, an integer or an `a-b` range, into the object list or,
    /// for a node command, the node list. Repeatable; unions with `--select`.
    #[arg(value_name = "select-index", long, value_parser = parse_index_range)]
    select_index: Vec<IndexRange>,
}

impl RequiredSelection {
    /// The ids of the objects the selectors match in `main`, in document
    /// order. Errors when they match nothing.
    pub fn resolve_objects<T: VoxExt>(&self, main: &VoxMain<T>) -> Result<Vec<U32Id<BVoxObject>>> {
        let object_ids = select_objects(main, &self.select, &self.select_index)?;

        if object_ids.is_empty() {
            return Err(Error::usage(
                "no object matched the selection; check --select and --select-index",
            ));
        }

        Ok(object_ids)
    }

    /// The ids of the nodes the selectors match in `main`, in document order.
    /// Errors when they match nothing.
    #[cfg_attr(not(test), expect(dead_code, reason = "node commands use it from S6"))]
    pub fn resolve_nodes<T: VoxExt>(
        &self,
        main: &VoxMain<T>,
    ) -> Result<Vec<U32Id<BVoxHierarchyNode>>> {
        let node_ids = select_nodes(main, &self.select, &self.select_index)?;

        if node_ids.is_empty() {
            return Err(Error::usage(
                "no node matched the selection; check --select and --select-index",
            ));
        }

        Ok(node_ids)
    }
}

#[cfg(test)]
mod tests {
    use crate::RequiredSelection;
    use clap::Parser;
    use voxcore::{VoxHierarchyNode, VoxMain};

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        selection: RequiredSelection,
    }

    fn selection(args: &[&str]) -> RequiredSelection {
        let mut argv = vec!["cli"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).unwrap().selection
    }

    #[test]
    fn a_selector_is_required() {
        assert!(Cli::try_parse_from(["cli"]).is_err());
        assert!(Cli::try_parse_from(["cli", "--select", "a", "--select-index", "0"]).is_ok());
    }

    #[test]
    fn a_selection_matching_nothing_is_an_error() {
        let mut main: VoxMain = VoxMain::default();

        let node = VoxHierarchyNode {
            name: "door".to_owned(),
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap();

        assert!(
            selection(&["--select", "door"])
                .resolve_objects(&main)
                .is_err()
        );
        assert!(
            selection(&["--select", "wall"])
                .resolve_nodes(&main)
                .is_err()
        );
        assert_eq!(
            selection(&["--select", "door"])
                .resolve_nodes(&main)
                .unwrap()
                .len(),
            1
        );
    }
}
