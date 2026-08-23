use crate::commands::mv_copy::{copy_dir_recursive, copy_permissions};
use crate::utils::moe::is_moe;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

#[inline]
fn format_mv_error(path: &Path, is_dir: bool) -> String {
    let path_display = path.display().to_string();
    if is_moe() {
        format!(
            "{} {} {} does not exist: {}",
            "😢💔".truecolor(255, 105, 180),
            "Error:".truecolor(255, 105, 180),
            if is_dir {
                "Destination directory"
            } else {
                "Destination parent directory"
            },
            path_display.truecolor(255, 182, 193)
        )
    } else {
        format!(
            "{} {} {} does not exist: {}",
            "❌".red(),
            "Error:".red(),
            if is_dir {
                "Destination directory"
            } else {
                "Destination parent directory"
            },
            path_display.bright_red()
        )
    }
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
            return Err(format_mv_error(&dest_path, true).into());
        }
        dest_path.join(source_path.file_name().ok_or("Invalid source path")?)
    } else {
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() && parent != Path::new("") {
                return Err(format_mv_error(parent, false).into());
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

    let source_display = source_path.display().to_string();
    let dest_display = final_dest.display().to_string();

    let output = if copy {
        if source_metadata.is_dir() {
            copy_dir_recursive(&source_path, &final_dest)?;
            format!(
                "{} Copied directory {} to {}",
                "✔".bright_green(),
                source_display.cyan(),
                dest_display.cyan()
            )
        } else {
            fs::copy(&source_path, &final_dest)?;
            copy_permissions(&source_path, &final_dest)?;

            format!(
                "{} Copied file {} to {}",
                "✔".bright_green(),
                source_display.cyan(),
                dest_display.cyan()
            )
        }
    } else {
        fs::rename(&source_path, &final_dest)?;
        format!(
            "{} Moved {} to {}",
            "✔".bright_green(),
            source_display.cyan(),
            dest_display.cyan()
        )
    };

    Ok((output, dest_display))
}

#[cfg(test)]
mod tests;
