use crossterm::{
    event::{poll, read},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use std::time::Duration;

/// Runs `print`, which writes terminal graphics escape sequences, with raw mode
/// asserted around it so capability-probe replies (Kitty graphics ACKs, Primary
/// Device Attributes, etc.) are not echoed back as visible garbage. viuer
/// toggles raw mode internally, so it is re-asserted afterwards and any
/// straggling reply bytes are drained before the original mode is restored.
pub fn with_capability_probe_guard<T>(print: impl FnOnce() -> T) -> T {
    let was_raw = is_raw_mode_enabled().unwrap_or(false);
    let _ = enable_raw_mode();

    let result = print();

    let _ = enable_raw_mode();
    while matches!(poll(Duration::from_millis(50)), Ok(true)) {
        if read().is_err() {
            break;
        }
    }
    if !was_raw {
        let _ = disable_raw_mode();
    }

    result
}
