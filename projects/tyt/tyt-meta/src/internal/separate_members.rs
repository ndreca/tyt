/// Puts a blank line between the members of the brace block whose opening line
/// matches `header`, so an inserted field, variant, or item follows the
/// workspace's spacing. A member's attributes and comments stay attached to it.
pub fn separate_members(lines: &mut Vec<String>, header: impl Fn(&str) -> bool) {
    let Some(open) = lines.iter().position(|line| header(line.trim())) else {
        return;
    };
    let mut depth = bracket_delta(&lines[open]);
    if depth != 1 {
        return;
    }

    let mut member_ended = false;
    let mut index = open + 1;
    while index < lines.len() {
        let trimmed = lines[index].trim().to_string();
        let comment = trimmed.starts_with("//");
        let delta = if comment { 0 } else { bracket_delta(&trimmed) };
        if depth + delta <= 0 {
            return;
        }

        if depth == 1 && member_ended && !trimmed.is_empty() {
            if !lines[index - 1].trim().is_empty() {
                lines.insert(index, String::new());
                index += 1;
            }
            member_ended = false;
        }

        depth += delta;
        if depth == 1 && !comment && trimmed.ends_with([',', ';', '}']) {
            member_ended = true;
        }
        index += 1;
    }
}

/// The net count of opening over closing brackets in `line`, skipping string
/// literals.
fn bracket_delta(line: &str) -> i32 {
    let mut delta = 0;
    let mut in_string = false;
    let mut escaped = false;
    for character in line.chars() {
        if in_string {
            match character {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '(' | '[' | '{' => delta += 1,
            ')' | ']' | '}' => delta -= 1,
            _ => {}
        }
    }
    delta
}

#[cfg(test)]
mod tests {
    use crate::separate_members;

    fn separated(source: &str, header: &str) -> String {
        let mut lines: Vec<String> = source.lines().map(str::to_string).collect();
        separate_members(&mut lines, |line| line.starts_with(header));
        lines.join("\n")
    }

    #[test]
    fn separates_variants_keeping_attributes_attached() {
        let source = "pub enum Tool {\n    #[command(name = \"a\")]\n    A(A),\n    #[command(name = \"b\")]\n    B(B),\n}";
        assert_eq!(
            separated(source, "pub enum Tool"),
            "pub enum Tool {\n    #[command(name = \"a\")]\n    A(A),\n\n    #[command(name = \"b\")]\n    B(B),\n}"
        );
    }

    #[test]
    fn separates_multiline_items_and_leaves_their_bodies_alone() {
        let source = "impl Deps for Impl {\n    type A = A;\n    fn a(&self) -> A {\n        A\n    }\n    fn b(&self) -> B {\n        B\n    }\n}";
        assert_eq!(
            separated(source, "impl Deps"),
            "impl Deps for Impl {\n    type A = A;\n\n    fn a(&self) -> A {\n        A\n    }\n\n    fn b(&self) -> B {\n        B\n    }\n}"
        );
    }

    #[test]
    fn already_separated_members_are_unchanged() {
        let source = "pub trait Deps {\n    type A: A;\n\n    fn a(&self) -> Self::A;\n}";
        assert_eq!(separated(source, "pub trait Deps"), source);
    }
}
