use branded_id::U32Id;
use std::collections::HashSet;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

/// Every hierarchy node's paths, each a chain of node names from a root. A
/// node reached through several parents gets one path per placement. A node
/// that neither the roots nor any node lists starts its own chain, the way an
/// unplaced object's path is its bare name.
pub fn node_paths<T: VoxExt>(main: &VoxMain<T>) -> Vec<(NodeId, String)> {
    let mut paths = Vec::new();

    let mut stack = Vec::new();

    for &root_id in main.root_hierarchy_node_ids() {
        walk_node_paths(main, root_id, "", &mut stack, &mut paths);
    }

    let listed: HashSet<NodeId> = main
        .root_hierarchy_node_ids()
        .iter()
        .copied()
        .chain(
            main.iter_hierarchy_nodes()
                .flat_map(|(_, node)| node.child_node_ids.iter().copied()),
        )
        .collect();

    let unplaced_ids: Vec<NodeId> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .filter(|node_id| !listed.contains(node_id))
        .collect();

    for node_id in unplaced_ids {
        walk_node_paths(main, node_id, "", &mut stack, &mut paths);
    }

    paths
}

/// Records `node_id`'s path (built from `prefix`), then recurses into its child
/// nodes. `stack` is the current root-to-node chain and guards against a cycle.
fn walk_node_paths<T: VoxExt>(
    main: &VoxMain<T>,
    node_id: NodeId,
    prefix: &str,
    stack: &mut Vec<NodeId>,
    paths: &mut Vec<(NodeId, String)>,
) {
    if stack.contains(&node_id) {
        return;
    }

    let Some(node) = main.hierarchy_node(node_id) else {
        return;
    };

    let path = if prefix.is_empty() {
        node.name.clone()
    } else {
        format!("{prefix}/{}", node.name)
    };

    paths.push((node_id, path.clone()));

    stack.push(node_id);

    for &child_id in &node.child_node_ids {
        walk_node_paths(main, child_id, &path, stack, paths);
    }

    stack.pop();
}

#[cfg(test)]
mod tests {
    use crate::utilities::node_paths::{NodeId, node_paths};
    use voxcore::{VoxHierarchyNode, VoxMain};

    fn node_id(main: &mut VoxMain, name: &str, child_node_ids: Vec<NodeId>) -> NodeId {
        let node = VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids,
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap()
    }

    fn paths(main: &VoxMain) -> Vec<String> {
        node_paths(main).into_iter().map(|(_, path)| path).collect()
    }

    #[test]
    fn a_shared_node_gets_one_path_per_placement() {
        let mut main = VoxMain::default();

        let shared_id = node_id(&mut main, "shared", vec![]);
        let left_id = node_id(&mut main, "left", vec![shared_id]);
        let right_id = node_id(&mut main, "right", vec![shared_id]);
        main.push_root_hierarchy_node_id(left_id).unwrap();
        main.push_root_hierarchy_node_id(right_id).unwrap();

        assert_eq!(
            paths(&main),
            ["left", "left/shared", "right", "right/shared"]
        );
    }

    #[test]
    fn an_unplaced_node_starts_its_own_chain() {
        let mut main = VoxMain::default();

        let child_id = node_id(&mut main, "child", vec![]);
        node_id(&mut main, "loose", vec![child_id]);
        let root_id = node_id(&mut main, "root", vec![]);
        main.push_root_hierarchy_node_id(root_id).unwrap();

        assert_eq!(paths(&main), ["root", "loose", "loose/child"]);
    }
}
