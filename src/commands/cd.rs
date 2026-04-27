use colored::*;
use std::env;
use std::path::PathBuf;

pub fn cmd_cd(path: Option<&str>) -> Result<(String, String), Box<dyn std::error::Error>> {
    let target = match path {
        Some("..") => {
            let mut current = env::current_dir()?;
            current.pop();
            current
        }
        Some("~") | None => dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        Some(p) => PathBuf::from(p),
    };

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    if !target.is_dir() {
        return Err(format!("Not a directory: {}", target.display()).into());
    }

    env::set_current_dir(&target)?;
    let plain_path = target.display().to_string();
    let display = format!("{} {}", "Changed to:".green(), plain_path.cyan());
    Ok((display, plain_path))
}