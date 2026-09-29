use crate::temp_counter_next;
use std::{
    env,
    io::{Error as IOError, Result},
    path::PathBuf,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

/// A fresh path under the system temp directory.
pub fn unique_temp_path() -> Result<PathBuf> {
    let mut base = env::temp_dir();

    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(IOError::other)?
        .as_nanos();

    let pid = process::id();
    let n = temp_counter_next();

    base.push(format!("tyt-{}-{}-{}", pid, now_ns, n));
    Ok(base)
}
