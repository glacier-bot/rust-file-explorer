use colored::*;
use std::path::PathBuf;
use std::process::Command;

pub fn cmd_open(path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let target = PathBuf::from(path);

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    let plain_path = target.display().to_string();
    
    if target.is_dir() {
        #[cfg(target_os = "windows")]
        Command::new("explorer.exe")
            .arg(&target)
            .spawn()?;
        
        #[cfg(target_os = "macos")]
        Command::new("open")
            .arg(&target)
            .spawn()?;
        
        #[cfg(target_os = "linux")]
        Command::new("xdg-open")
            .arg(&target)
            .spawn()?;
        
        let display = format!(
            "{} {} {}",
            "✔ Opened directory".bright_green(),
            plain_path.cyan(),
            "in file explorer".bright_green()
        );
        return Ok((display, plain_path));
    }

    open::that(&target)?;
    let display = format!(
        "{} {} {}",
        "✔ Opened file".bright_green(),
        plain_path.cyan(),
        "with default application".bright_green()
    );
    Ok((display, plain_path))
}