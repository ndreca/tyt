/// Fields from a failed external command execution.
#[derive(Debug)]
pub struct ExecFailed {
    /// The process exit code, or `None` when a signal ended the process.
    pub exit_code: Option<i32>,

    /// The captured standard output.
    pub stdout: String,

    /// The captured standard error.
    pub stderr: String,
}
