use crossterm::{
    event::{poll, read},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use std::{io::Result, time::Duration};

/// Runs `print`, which writes terminal graphics escape sequences, with raw mode
/// asserted around it so capability-probe replies (Kitty graphics ACKs, Primary
/// Device Attributes, etc.) are not echoed back as visible garbage. viuer
/// toggles raw mode internally, so it is re-asserted afterwards and any
/// straggling reply bytes are drained before the original mode is restored.
pub fn with_capability_probe_guard<T>(print: impl FnOnce() -> Result<T>) -> Result<T> {
    let was_raw = is_raw_mode_enabled()?;
    enable_raw_mode()?;

    let result = print();

    let drained = enable_raw_mode().and_then(|()| drain_replies());
    let restored = if was_raw { Ok(()) } else { disable_raw_mode() };

    let value = result?;
    drained?;
    restored?;

    Ok(value)
}

/// Reads and discards input until none arrives for 50 milliseconds.
fn drain_replies() -> Result<()> {
    while poll(Duration::from_millis(50))? {
        read()?;
    }

    Ok(())
}
