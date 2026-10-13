use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

pub(crate) fn inspect_input(
    path: &Path,
    owner: u32,
    count: &mut usize,
    bytes: &mut u64,
    depth: usize,
) -> Result<(), String> {
    if depth > 8 {
        return Err("input depth exceeds bounds".into());
    }
    for item in fs::read_dir(path).map_err(|_| "input unavailable")? {
        let item = item.map_err(|_| "input unavailable")?;
        let meta = fs::symlink_metadata(item.path()).map_err(|_| "input unavailable")?;
        *count += 1;
        *bytes = bytes.saturating_add(meta.len());
        if *count > 512
            || *bytes > 4 * 1024 * 1024
            || meta.uid() != owner
            || meta.permissions().mode() & 0o022 != 0
            || (!meta.is_dir() && !meta.is_file())
            || (meta.is_file() && meta.nlink() != 1)
        {
            return Err("input must be bounded owner-controlled regular files/directories".into());
        }
        if meta.is_dir() {
            inspect_input(&item.path(), owner, count, bytes, depth + 1)?;
        }
    }
    Ok(())
}
