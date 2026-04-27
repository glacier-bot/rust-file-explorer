use arboard::Clipboard;
use colored::*;
use std::env;
use std::fs;
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

    let abs_path = fs::canonicalize(&path)
        .map_err(|e| format!("failed to resolve absolute path: {}", e))?;

    let path_str = if cfg!(windows) {
        let s = abs_path.to_string_lossy().to_string();
        s.strip_prefix(r"\\?\").map(|stripped| stripped.to_string()).unwrap_or(s)
    } else {
        abs_path.to_string_lossy().to_string()
    };

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("failed to access system clipboard: {}", e))?;
    clipboard
        .set_text(&path_str)
        .map_err(|e| format!("failed to copy to clipboard: {}", e))?;

    let display = format!("{} {}", "✔ Copied to clipboard:".bright_green(), path_str.cyan());
    Ok((display, path_str))
}