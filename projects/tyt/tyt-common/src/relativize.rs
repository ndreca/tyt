use std::path::{Component, Path, PathBuf};

/// Re-expresses `target` relative to `base` after normalizing both lexically. A
/// `target` sharing no root with `base` comes back only normalized.
pub fn relativize(base: &Path, target: &Path) -> PathBuf {
    let base = normalize(base);
    let target = normalize(target);

    let base_components: Vec<Component> = base.components().collect();
    let target_components: Vec<Component> = target.components().collect();

    let common = base_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(b, t)| b == t)
        .count();

    let base_rest = &base_components[common..];
    let target_rest = &target_components[common..];

    // A leftover root or prefix in `base` means the two paths do not share a
    // root, so no `..`-prefixed relative path can reach `target`.
    if base_rest
        .iter()
        .any(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
    {
        return target;
    }

    let mut result = PathBuf::new();
    for _ in base_rest {
        result.push("..");
    }
    for component in target_rest {
        result.push(component.as_os_str());
    }

    if result.as_os_str().is_empty() {
        result.push(".");
    }
    result
}

/// Resolves `.` and `..` components without consulting the filesystem. A `..`
/// that climbs above a relative path's start stays in the result.
fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}

            Component::ParentDir => match result.components().next_back() {
                Some(Component::Normal(_)) => {
                    result.pop();
                }

                Some(Component::RootDir | Component::Prefix(_)) => {}

                _ => result.push(".."),
            },

            other => result.push(other.as_os_str()),
        }
    }
    result
}
