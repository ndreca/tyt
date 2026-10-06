/// Renders a new crate's `Cargo.toml` with its binary named `command`.
pub fn cargo_toml_template(package: &str, command: &str, description: &str) -> String {
    format!(
        r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2024"
license-file = "LICENSE"
repository = "https://github.com/tyleo/tyt"
description = "{description}"
autobins = false

[[bin]]
name = "{command}"
path = "src/main.rs"
required-features = ["bin"]

[dependencies]
clap = {{ version = "4.5.58", features = ["derive"] }}
tyt-clap = {{ version = "0.1.0", optional = true }}
tyt-common = {{ version = "0.2.0" }}
tyt-injection = {{ version = "0.3.0", optional = true }}

[features]
default = ["impl"]
impl = ["dep:tyt-injection"]
bin = ["impl", "dep:ty-clap"]
"#
    )
}
