use crate::commands::cd::CdSelectionItem;
use crate::managers::tag::TagManager;
use colored::*;
use regex::Regex;
use std::env;
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

fn find_files_by_tag(
    tag: &str,
    tag_manager: &TagManager,
) -> Result<Vec<CdSelectionItem>, Box<dyn std::error::Error>> {
    let tag_regex = Regex::new(tag)?;
    let current_dir = env::current_dir()?;
    let mut matching_files = Vec::new();

    for (path, tags) in tag_manager.list_all() {
        if tags.iter().any(|t| tag_regex.is_match(t)) {
            let mut full_path = PathBuf::from(path);
            let path_lower = path.to_lowercase();
            let is_index_file = path_lower.contains(".index") || path_lower.ends_with("index");
            if is_index_file {
                full_path.pop();
            }

            let display_path = match full_path.strip_prefix(&current_dir) {
                Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                Err(_) => full_path.to_string_lossy().to_string(),
            };

            matching_files.push(CdSelectionItem {
                display_path,
                full_path: full_path.to_string_lossy().to_string(),
                tags: tags.clone(),
            });
        }
    }

    if matching_files.is_empty() {
        return Err(format!("No files found matching tag: {}", tag).into());
    }

    Ok(matching_files)
}

pub fn cmd_open_tag(
    tag: Option<&str>,
    tag_manager: Option<&TagManager>,
    selection: Option<usize>,
) -> Result<OpenResult, Box<dyn std::error::Error>> {
    let tag = tag.ok_or("Usage: open -tag <tag>")?;
    let tag_manager = tag_manager.ok_or("Tag manager not available")?;
    let matching_files = find_files_by_tag(tag, tag_manager)?;

    if matching_files.len() == 1 {
        let item = &matching_files[0];
        let (display, raw) = cmd_open(&item.full_path)?;
        return Ok(OpenResult::Success(display, raw));
    }

    if let Some(sel) = selection {
        if sel < 1 || sel > matching_files.len() {
            return Err(format!(
                "Invalid selection. Please enter a number between 1 and {}",
                matching_files.len()
            )
            .into());
        }

        let item = &matching_files[sel - 1];
        let (display, raw) = cmd_open(&item.full_path)?;
        return Ok(OpenResult::Success(display, raw));
    }

    Ok(OpenResult::NeedSelection(matching_files))
}
