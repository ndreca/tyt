/// Converts a kebab-case name to snake_case (e.g., `foo-bar` → `foo_bar`).
pub fn kebab_to_snake_case(command: &str) -> String {
    command.replace('-', "_")
}
