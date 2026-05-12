use colored::*;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use crate::managers::tag::TagManager;

#[derive(Debug, Clone)]
pub struct CdSelectionItem {
    pub display_path: String,
    pub full_path: String,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub enum CdResult {
    Success(String, String, Option<String>),
    NeedSelection(Vec<CdSelectionItem>),
}

pub fn cmd_cd(
    path: Option<&str>, 
    previous_dir: Option<&str>,
    is_idx: bool,
    tag: Option<&str>,
    tag_manager: Option<&TagManager>,
    selection: Option<usize>,
) -> Result<CdResult, Box<dyn std::error::Error>> {
    if is_idx {
        return cmd_cd_idx(tag, tag_manager, selection);
    }

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
    Ok(CdResult::Success(display, plain_path, new_previous_dir))
}

fn cmd_cd_idx(
    tag: Option<&str>,
    tag_manager: Option<&TagManager>,
    selection: Option<usize>,
) -> Result<CdResult, Box<dyn std::error::Error>> {
    let tag = tag.ok_or("Usage: cd -idx <tag>")?;
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
        let target = PathBuf::from(&item.full_path);
        let current_dir = env::current_dir()?;
        
        let new_previous_dir = if target != current_dir {
            Some(current_dir.display().to_string())
        } else {
            None
        };
        
        env::set_current_dir(&target)?;
        let plain_path = target.display().to_string();
        let display = format!("{} {}", "Changed to:".green(), plain_path.cyan());
        return Ok(CdResult::Success(display, plain_path, new_previous_dir));
    }
    
    if let Some(sel) = selection {
        if sel < 1 || sel > matching_dirs.len() {
            return Err(format!("Invalid selection. Please enter a number between 1 and {}", matching_dirs.len()).into());
        }
        
        let item = &matching_dirs[sel - 1];
        let target = PathBuf::from(&item.full_path);
        
        if !target.exists() {
            return Err(format!("Directory does not exist or is not accessible: {}", target.display()).into());
        }
        
        let current_dir = env::current_dir()?;
        
        let new_previous_dir = if target != current_dir {
            Some(current_dir.display().to_string())
        } else {
            None
        };
        
        env::set_current_dir(&target)?;
        let plain_path = target.display().to_string();
        let display = format!("{} {}", "Changed to:".green(), plain_path.cyan());
        return Ok(CdResult::Success(display, plain_path, new_previous_dir));
    }
    
    Ok(CdResult::NeedSelection(matching_dirs))
}

pub fn render_selection_list(items: &[CdSelectionItem]) -> String {
    let mut output = format!("{} Multiple directories found:\n\n", "🔍".yellow().bold());
    for (i, item) in items.iter().enumerate() {
        output.push_str(&format!(
            "  {}. {} -> {}\n",
            (i + 1).to_string().bright_blue(),
            item.display_path.cyan(),
            item.tags.join(", ").bright_yellow()
        ));
    }
    output
}
