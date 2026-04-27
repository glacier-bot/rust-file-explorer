use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&entry_path)?;
                fs::set_permissions(&dest_path, metadata.permissions())?;
            }
            
            #[cfg(windows)]
            {
                let metadata = fs::metadata(&entry_path)?;
                let mut perm = fs::metadata(&dest_path)?.permissions();
                perm.set_readonly(metadata.permissions().readonly());
                fs::set_permissions(&dest_path, perm)?;
            }
        }
    }

    Ok(())
}

pub fn cmd_mv(source: &str, destination: &str, copy: bool) -> Result<(String, String), Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(source);
    let dest_path = PathBuf::from(destination);

    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source_path.display()).into());
    }

    let source_metadata = fs::metadata(&source_path)?;

    let final_dest = if dest_path.is_dir() {
        dest_path.join(source_path.file_name().ok_or("Invalid source path")?)
    } else {
        dest_path.clone()
    };

    if final_dest.exists() {
        return Err(format!("Destination path already exists: Please remove it first: {}", final_dest.display()).into());
    }

    let output = if copy {
        if source_metadata.is_dir() {
            copy_dir_recursive(&source_path, &final_dest)?;
            format!(
                "{} Copied directory {} to {}",
                "✔".bright_green(),
                source_path.display().to_string().cyan(),
                final_dest.display().to_string().cyan()
            )
        } else {
            fs::copy(&source_path, &final_dest)?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perm = fs::metadata(&final_dest)?.permissions();
                perm.set_mode(source_metadata.permissions().mode());
                fs::set_permissions(&final_dest, perm)?;
            }
            
            #[cfg(windows)]
            {
                let mut perm = fs::metadata(&final_dest)?.permissions();
                perm.set_readonly(source_metadata.permissions().readonly());
                fs::set_permissions(&final_dest, perm)?;
            }

            format!(
                "{} Copied file {} to {}",
                "✔".bright_green(),
                source_path.display().to_string().cyan(),
                final_dest.display().to_string().cyan()
            )
        }
    } else {
        fs::rename(&source_path, &final_dest)?;
        format!(
            "{} Moved {} to {}",
            "✔".bright_green(),
            source_path.display().to_string().cyan(),
            final_dest.display().to_string().cyan()
        )
    };

    let raw_path = final_dest.display().to_string();
    Ok((output, raw_path))
}