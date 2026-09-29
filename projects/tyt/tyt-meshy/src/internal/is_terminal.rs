use crate::MeshTask;

/// Whether a task has reached a terminal status (successfully or not).
pub fn is_terminal(task: &MeshTask) -> bool {
    matches!(task.status.as_str(), "SUCCEEDED" | "FAILED" | "CANCELED")
}
