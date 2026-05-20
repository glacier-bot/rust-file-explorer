use crate::utils::moe::is_moe;
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

pub fn cmd_mv(
    source: &str,
    destination: &str,
    copy: bool,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(source);
    let dest_path = PathBuf::from(destination);

    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source_path.display()).into());
    }

    let source_metadata = fs::metadata(&source_path)?;

    let dest_is_dir =
        destination.ends_with('/') || destination.ends_with('\\') || dest_path.is_dir();

    let final_dest = if dest_is_dir {
        if !dest_path.exists() {
            let error_msg = if is_moe() {
                format!(
                    "{} {} Destination directory does not exist: {}",
                    "😢💔".truecolor(255, 105, 180),
                    "Error:".truecolor(255, 105, 180),
                    dest_path.display().to_string().truecolor(255, 182, 193)
                )
            } else {
                format!(
                    "{} {} {}",
                    "❌".red(),
                    "Error:".red(),
                    format!(
                        "Destination directory does not exist: {}",
                        dest_path.display()
                    )
                    .bright_red()
                )
            };
            return Err(error_msg.into());
        }
        dest_path.join(source_path.file_name().ok_or("Invalid source path")?)
    } else {
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() && parent != Path::new("") {
                let error_msg = if is_moe() {
                    format!(
                        "{} {} Destination parent directory does not exist: {}",
                        "😢💔".truecolor(255, 105, 180),
                        "Error:".truecolor(255, 105, 180),
                        parent.display().to_string().truecolor(255, 182, 193)
                    )
                } else {
                    format!(
                        "{} {} {}",
                        "❌".red(),
                        "Error:".red(),
                        format!(
                            "Destination parent directory does not exist: {}",
                            parent.display()
                        )
                        .bright_red()
                    )
                };
                return Err(error_msg.into());
            }
        }
        dest_path.clone()
    };

    if final_dest.exists() {
        return Err(format!(
            "Destination path already exists: Please remove it first: {}",
            final_dest.display()
        )
        .into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::moe::{enable_moe, disable_moe};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_mv_file_to_nonexistent_dir_returns_error() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test_file.txt");
        fs::write(&source_path, "test content").unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            "nonexistent_dir/",
            false,
        );

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Destination directory does not exist"));
        assert!(source_path.exists());
    }

    #[test]
    fn test_mv_file_with_nonexistent_parent_dir_returns_error() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test_file.txt");
        fs::write(&source_path, "test content").unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            "nonexistent_parent/new_file.txt",
            false,
        );

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Destination parent directory does not exist"));
        assert!(source_path.exists());
    }

    #[test]
    fn test_mv_file_to_existing_dir() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test_file.txt");
        let dest_dir = temp_dir.path().join("existing_dir");
        fs::write(&source_path, "test content").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            dest_dir.to_str().unwrap(),
            false,
        );

        assert!(result.is_ok());
        let (output, raw) = result.unwrap();
        assert!(output.contains("Moved"));
        assert!(raw.contains("existing_dir") && raw.contains("test_file.txt"));
        assert!(!source_path.exists());
        assert!(dest_dir.join("test_file.txt").exists());
    }

    #[test]
    fn test_mv_rename_file() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("old_name.txt");
        let dest_path = temp_dir.path().join("new_name.txt");
        fs::write(&source_path, "test content").unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
            false,
        );

        assert!(result.is_ok());
        let (output, raw) = result.unwrap();
        assert!(output.contains("Moved"));
        assert!(raw.contains("new_name.txt"));
        assert!(!source_path.exists());
        assert!(dest_path.exists());
    }

    #[test]
    fn test_mv_destination_already_exists_returns_error() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("file_a.txt");
        let dest_path = temp_dir.path().join("file_b.txt");
        fs::write(&source_path, "content a").unwrap();
        fs::write(&dest_path, "content b").unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            dest_path.to_str().unwrap(),
            false,
        );

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Destination path already exists"));
        assert!(source_path.exists());
        assert!(dest_path.exists());
    }

    #[test]
    fn test_mv_source_not_exists_returns_error() {
        let result = cmd_mv("nonexistent_source.txt", "some_dir/", false);

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Source path does not exist"));
    }

    #[test]
    fn test_mv_copy_mode_to_nonexistent_dir_returns_error() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test_file.txt");
        fs::write(&source_path, "test content").unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            "nonexistent_dir/",
            true,
        );

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Destination directory does not exist"));
        assert!(source_path.exists());
    }

    #[test]
    fn test_mv_copy_mode_to_existing_dir() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test_file.txt");
        let dest_dir = temp_dir.path().join("existing_dir");
        fs::write(&source_path, "test content").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            dest_dir.to_str().unwrap(),
            true,
        );

        assert!(result.is_ok());
        let (output, raw) = result.unwrap();
        assert!(output.contains("Copied"));
        assert!(source_path.exists());
        assert!(dest_dir.join("test_file.txt").exists());
    }

    #[test]
    fn test_mv_error_message_std_mode() {
        disable_moe();
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test_file.txt");
        fs::write(&source_path, "test content").unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            "nonexistent_dir/",
            false,
        );

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("Error:"));
        assert!(err_msg.contains("Destination directory does not exist"));
    }

    #[test]
    fn test_mv_error_message_moe_mode() {
        enable_moe();
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test_file.txt");
        fs::write(&source_path, "test content").unwrap();

        let result = cmd_mv(
            source_path.to_str().unwrap(),
            "nonexistent_dir/",
            false,
        );

        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("😢"));
        assert!(err_msg.contains("💔"));
        assert!(err_msg.contains("Error:"));
        assert!(err_msg.contains("Destination directory does not exist"));
        disable_moe();
    }

    #[test]
    fn test_mv_directory_to_existing_dir() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source_dir");
        let dest_dir = temp_dir.path().join("dest_dir");
        fs::create_dir(&source_dir).unwrap();
        fs::write(source_dir.join("file.txt"), "content").unwrap();
        fs::create_dir(&dest_dir).unwrap();

        let result = cmd_mv(
            source_dir.to_str().unwrap(),
            dest_dir.to_str().unwrap(),
            false,
        );

        assert!(result.is_ok());
        let (output, raw) = result.unwrap();
        assert!(output.contains("Moved"));
        assert!(!source_dir.exists());
        assert!(dest_dir.join("source_dir").exists());
        assert!(dest_dir.join("source_dir").join("file.txt").exists());
    }
}
