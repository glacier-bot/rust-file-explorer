use colored::*;
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;
use std::env;
use crate::managers::tag::TagManager;
use crate::commands::cd::CdSelectionItem;

pub fn cmd_open(path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let target = PathBuf::from(path);

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    let plain_path = target.display().to_string();

    if target.is_dir() {
        #[cfg(target_os = "windows")]
        Command::new("explorer.exe").arg(&target).spawn()?;

        #[cfg(target_os = "macos")]
        Command::new("open").arg(&target).spawn()?;

        #[cfg(target_os = "linux")]
        Command::new("xdg-open").arg(&target).spawn()?;

        let display = format!(
            "{} {} {}",
            "✔ Opened directory".bright_green(),
            plain_path.cyan(),
            "in file explorer".bright_green()
        );
        return Ok((display, plain_path));
    }

    #[cfg(target_os = "windows")]
    {
        // 安全地打开文件：使用 Start-Process 的 -FilePath 指定路径，
        // 路径用英文双引号包裹，防止 PowerShell 解析空格、括号等特殊字符
        let path_str = target.to_string_lossy().to_string();
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Start-Process -FilePath \"{}\"",
                    path_str.replace('"', "`\""),
                ),
            ])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    Command::new("open").arg(&target).spawn()?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(&target).spawn()?;

    let display = format!(
        "{} {} {}",
        "✔ Opened file".bright_green(),
        plain_path.cyan(),
        "with default application".bright_green()
    );
    Ok((display, plain_path))
}

#[derive(Debug)]
pub enum OpenResult {
    Success(String, String),
    NeedSelection(Vec<CdSelectionItem>),
}

pub fn cmd_open_tag(
    tag: Option<&str>,
    tag_manager: Option<&TagManager>,
    selection: Option<usize>,
) -> Result<OpenResult, Box<dyn std::error::Error>> {
    let tag = tag.ok_or("Usage: open -tag <tag>")?;
    let tag_manager = tag_manager.ok_or("Tag manager not available")?;

    let tag_regex = Regex::new(tag)?;
    
    let mut matching_dirs = Vec::new();
    
    for (path, tags) in tag_manager.list_all() {
        let is_index_file = path.to_lowercase().contains(".index") 
            || path.to_lowercase().ends_with("index");
        if is_index_file && tags.iter().any(|t| tag_regex.is_match(t)) {
            let mut dir_path = PathBuf::from(path);
            dir_path.pop();
            
            let full_path = dir_path.to_string_lossy().to_string();
            
            let current_dir = env::current_dir()?;
            let display_path = match dir_path.strip_prefix(&current_dir) {
                Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                Err(_) => full_path.clone(),
            };
            
            matching_dirs.push(CdSelectionItem {
                display_path,
                full_path,
                tags: tags.clone(),
            });
        }
    }
    
    if matching_dirs.is_empty() {
        return Err(format!("No directories found with .index file matching tag: {}", tag).into());
    }
    
    if matching_dirs.len() == 1 {
        let item = &matching_dirs[0];
        let (display, raw) = cmd_open(&item.full_path)?;
        return Ok(OpenResult::Success(display, raw));
    }
    
    if let Some(sel) = selection {
        if sel < 1 || sel > matching_dirs.len() {
            return Err(format!("Invalid selection. Please enter a number between 1 and {}", matching_dirs.len()).into());
        }
        
        let item = &matching_dirs[sel - 1];
        let (display, raw) = cmd_open(&item.full_path)?;
        return Ok(OpenResult::Success(display, raw));
    }
    
    Ok(OpenResult::NeedSelection(matching_dirs))
}
