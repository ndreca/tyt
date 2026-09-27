use crate::{Error, Result, parse_index_range};
use branded_id::U32Id;
use clap::Args;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxMain};
use voxsmith::utilities::{IndexRange, select_nodes};

/// The `--select-parent` / `--select-parent-index` selectors, which pick the
/// parent end of an edge.
#[derive(Clone, Debug, Args)]
pub struct ParentSelection {
    /// Choose the parent node by hierarchy-path glob, matched as a node
    /// command's `--select` matches. Repeatable; unions with
    /// `--select-parent-index`.
    #[arg(value_name = "select-parent", long)]
    select_parent: Vec<String>,

    /// Choose the parent node by index into the node list, an integer or an
    /// `a-b` range. Repeatable; unions with `--select-parent`.
    #[arg(value_name = "select-parent-index", long, value_parser = parse_index_range)]
    select_parent_index: Vec<IndexRange>,
}

impl ParentSelection {
    /// The parent node the selectors match in `main`. Returns `None` for the
    /// root list when no selector was given, and errors unless the selectors
    /// match exactly one node.
    #[cfg_attr(not(test), expect(dead_code, reason = "object link uses it from S4"))]
    pub fn resolve<T: VoxExt>(
        &self,
        main: &VoxMain<T>,
    ) -> Result<Option<U32Id<BVoxHierarchyNode>>> {
        if self.select_parent.is_empty() && self.select_parent_index.is_empty() {
            return Ok(None);
        }

        let node_ids = select_nodes(main, &self.select_parent, &self.select_parent_index)?;

        let [node_id] = node_ids[..] else {
            return Err(Error::usage(format!(
                "the parent selection matched {} nodes but must match exactly one; check \
                 --select-parent and --select-parent-index",
                node_ids.len()
            )));
        };

        Ok(Some(node_id))
    }
}

#[cfg(test)]
mod tests {
    use crate::ParentSelection;
    use clap::Parser;
    use voxcore::{VoxHierarchyNode, VoxMain};

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        parent: ParentSelection,
    }

    fn parent(args: &[&str]) -> ParentSelection {
        let mut argv = vec!["cli"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).unwrap().parent
    }

    /// Two root nodes, both named `door`.
    fn two_doors() -> VoxMain {
        let mut main = VoxMain::default();

        for _ in 0..2 {
            let node = VoxHierarchyNode {
                name: "door".to_owned(),
                ..Default::default()
            };

            let node_id = main.retain_hierarchy_node(node).unwrap();

            main.push_root_hierarchy_node_id(node_id).unwrap();
        }

        main
    }

    #[test]
    fn no_selector_targets_the_root_list() {
        assert_eq!(parent(&[]).resolve(&two_doors()).unwrap(), None);
    }

    #[test]
    fn the_selectors_must_match_exactly_one_node() {
        let main = two_doors();

        assert!(parent(&["--select-parent", "door"]).resolve(&main).is_err());
        assert!(parent(&["--select-parent", "wall"]).resolve(&main).is_err());
        assert!(
            parent(&["--select-parent-index", "1"])
                .resolve(&main)
                .unwrap()
                .is_some()
        );
    }
}
