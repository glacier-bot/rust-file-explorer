//! 命令模块
//! 包含各种命令的实现

use crate::commands::cd::CdSelectionItem;
use crate::managers::tag::TagManager;
use regex::Regex;
use std::env;
use std::path::PathBuf;

pub mod alias;
pub mod cd;
pub mod change;
pub mod clear;
pub mod cli;
pub mod clipboard;
pub mod help;
pub mod ls;
pub mod mkdf;
pub mod mv;
pub mod open;
pub mod pwd;
pub mod render;
pub mod shell;
pub mod tag;
pub mod welcome;

pub fn find_index_dirs_by_tag(
    tag: &str,
    tag_manager: &TagManager,
) -> Result<Vec<CdSelectionItem>, Box<dyn std::error::Error>> {
    let tag_regex = Regex::new(tag)?;
    let mut matching_dirs = Vec::new();

    for (path, tags) in tag_manager.list_all() {
        let path_lower = path.to_lowercase();
        let is_index_file = path_lower.contains(".index") || path_lower.ends_with("index");
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
        return Err(format!(
            "No directories found with .index file matching tag: {}",
            tag
        )
        .into());
    }

    Ok(matching_dirs)
}
