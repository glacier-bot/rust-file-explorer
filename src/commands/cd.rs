use colored::*;
use std::env;
use std::path::PathBuf;

pub fn cmd_cd(path: Option<&str>, previous_dir: Option<&str>) -> Result<(String, String, Option<String>), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    
    let target = match path {
        Some("-b") | Some("-back") => {
            if let Some(prev) = previous_dir {
                PathBuf::from(prev)
            } else {
                return Err("No previous directory available.".into());
            }
        }
        Some("..") => {
            let mut current = current_dir.clone();
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

    let new_previous_dir = if target != current_dir {
        Some(current_dir.display().to_string())
    } else {
        None
    };

    env::set_current_dir(&target)?;
    let plain_path = target.display().to_string();
    let display = format!("{} {}", "Changed to:".green(), plain_path.cyan());
    Ok((display, plain_path, new_previous_dir))
}