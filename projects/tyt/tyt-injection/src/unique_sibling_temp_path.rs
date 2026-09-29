use crate::temp_counter_next;
use std::{
    ffi::OsString,
    io::{Error as IOError, ErrorKind, Result},
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

/// A fresh hidden temp path beside `dst` for a write that renames over `dst`.
/// Errors when `dst` has no file name.
pub fn unique_sibling_temp_path(dst: &Path) -> Result<PathBuf> {
    let file_name = dst.file_name().ok_or_else(|| {
        IOError::new(
            ErrorKind::InvalidInput,
            format!("{} has no file name to write", dst.display()),
        )
    })?;
    let parent = dst.parent().expect("a path with a file name has a parent");

    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(IOError::other)?
        .as_nanos();

    let pid = process::id();
    let n = temp_counter_next();

    let mut tmp_name = OsString::from(".");
    tmp_name.push(file_name);
    tmp_name.push(format!(".tmp-{pid}-{now_ns}-{n}"));

    Ok(parent.join(tmp_name))
}

#[cfg(test)]
mod tests {
    use crate::unique_sibling_temp_path;
    use std::path::Path;

    #[test]
    fn a_path_without_a_file_name_is_an_error() {
        assert!(unique_sibling_temp_path(Path::new("/")).is_err());
        assert!(unique_sibling_temp_path(Path::new("dir/..")).is_err());
    }

    #[test]
    fn the_temp_path_sits_beside_the_destination_under_its_name() {
        let tmp = unique_sibling_temp_path(Path::new("dir/config.json")).unwrap();
        assert_eq!(tmp.parent(), Some(Path::new("dir")));
        let tmp_name = tmp.file_name().unwrap().to_str().unwrap();
        assert!(tmp_name.starts_with(".config.json.tmp-"), "{tmp_name}");
    }

    #[cfg(unix)]
    #[test]
    fn a_non_utf8_file_name_is_kept() {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

        let dst = Path::new(OsStr::from_bytes(b"dir/\xff.json"));
        let tmp = unique_sibling_temp_path(dst).unwrap();
        assert!(
            tmp.file_name()
                .unwrap()
                .as_bytes()
                .starts_with(b".\xff.json.tmp-")
        );
    }
}
