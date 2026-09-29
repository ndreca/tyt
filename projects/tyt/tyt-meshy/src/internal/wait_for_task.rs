use crate::{Dependencies, Error, MeshTask, OutputMode, Result, is_terminal};

/// Polls a task until it reaches a terminal status or the timeout elapses.
/// `get` fetches the task from its endpoint. Each non-terminal poll reports the
/// task's current progress through `output` (silent for the `id` and `quiet`
/// modes).
pub fn wait_for_task(
    dependencies: &impl Dependencies,
    get: impl Fn() -> Result<MeshTask>,
    output: OutputMode,
    task_id: &str,
    interval: u64,
    timeout: u64,
) -> Result<MeshTask> {
    let mut waited = 0u64;
    loop {
        let task = get()?;
        if is_terminal(&task) {
            return Ok(task);
        }
        if let Some(line) = output.poll_line(task_id, &task.status, task.progress) {
            dependencies.write_stdout(line.as_bytes())?;
        }
        if waited >= timeout {
            return Err(Error::PollTimeout(timeout));
        }
        dependencies.sleep(interval)?;
        waited += interval;
    }
}
