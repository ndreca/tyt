use crate::SceneNodeKind;
use voxcore::VoxHierarchyNode;

/// The kind a synthesized scene node takes from `node`'s children.
pub fn scene_node_kind_of(node: &VoxHierarchyNode) -> SceneNodeKind {
    if !node.child_object_ids.is_empty() {
        SceneNodeKind::Shape
    } else if node.child_node_ids.len() == 1 {
        SceneNodeKind::Transform
    } else {
        SceneNodeKind::Group
    }
}
