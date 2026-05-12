use arboard::Clipboard;
use colored::*;
use std::env;
use std::path::PathBuf;

pub fn cmd_cppwd() -> Result<(String, String), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let path_str = cwd.to_string_lossy().to_string();

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("failed to access system clipboard: {}", e))?;
    clipboard
        .set_text(&path_str)
        .map_err(|e| format!("failed to copy to clipboard: {}", e))?;

    let display = format!("{} {}", "✔ Copied to clipboard:".bright_green(), path_str.cyan());
    Ok((display, path_str))
}

pub fn cmd_cpf(file_path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let path = PathBuf::from(file_path);

    if !path.exists() {
        return Err(format!("file does not exist: {}", file_path).into());
    }

    if !path.is_file() {
        return Err(format!("not a file: {}", file_path).into());
    }

    let abs_path = if path.is_absolute() {
        path.clone()
    } else {
        env::current_dir()?.join(&path)
    };

    let path_str = abs_path.to_string_lossy().to_string();

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("failed to access system clipboard: {}", e))?;
    clipboard
        .set_text(&path_str)
        .map_err(|e| format!("failed to copy to clipboard: {}", e))?;

    let display = format!("{} {}", "✔ Copied to clipboard:".bright_green(), path_str.cyan());
    Ok((display, path_str))
}