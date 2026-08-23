use std::fs;
use std::path::Path;

pub(crate) fn copy_dir_recursive(
    source: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let entry_path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if entry_type.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
            copy_permissions(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

#[inline]
pub(crate) fn copy_permissions(
    source: &Path,
    dest: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(source)?;
        fs::set_permissions(dest, metadata.permissions())?;
    }

    #[cfg(windows)]
    {
        let metadata = fs::metadata(source)?;
        let mut perm = fs::metadata(dest)?.permissions();
        perm.set_readonly(metadata.permissions().readonly());
        fs::set_permissions(dest, perm)?;
    }

    Ok(())
}
