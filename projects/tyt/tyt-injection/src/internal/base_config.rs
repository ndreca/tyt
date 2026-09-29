use viuer::Config;

/// The base viuer config shared by every inline-image render: anchored at the
/// cursor rather than the terminal origin and, on Windows, skipping the
/// Kitty/iTerm2 capability probes that hang there.
pub fn base_config() -> Config {
    Config {
        absolute_offset: false,
        // No Windows terminal supports the Kitty graphics or iTerm2
        // inline-image protocols, but viuer still probes for them by writing
        // escape sequences and blocking on `read_key` for the reply. Windows
        // Terminal never answers, so the probe hangs until the user presses a
        // key. Skip the probes on Windows; Sixel and the ANSI fallback are
        // still tried.
        #[cfg(windows)]
        use_kitty: false,
        #[cfg(windows)]
        use_iterm: false,
        ..Default::default()
    }
}
