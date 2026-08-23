use colored::Color;
use regex::Regex;
use std::env;
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::commands::ls_icons::get_file_icon_and_color;
use crate::managers::tag::TagManager;
use crate::models::FileInfo;
use crate::utils::path::is_hidden;

#[inline]
fn is_inside_gdb(path: &Path) -> bool {
    path.ancestors().skip(1).any(|a| {
        a.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".gdb"))
    })
}

#[inline]
pub(crate) fn build_file_info(
    path: &Path,
    meta: &std::fs::Metadata,
    name: String,
    full_path: String,
    show_tags: bool,
    tag_manager: &TagManager,
) -> FileInfo {
    let (icon, color) = get_file_icon_and_color(path, meta);
    let tags = if show_tags {
        tag_manager.get_tags(path.to_str().unwrap_or(""))
    } else {
        Vec::new()
    };

    FileInfo {
        name,
        full_path,
        icon,
        color,
        size: meta.len(),
        created: meta.created().ok(),
        modified: meta.modified().unwrap_or_else(|_| SystemTime::now()),
        is_dir: meta.is_dir(),
        tags,
    }
}

#[inline]
pub(crate) fn build_error_file_info(name: String, full_path: String) -> FileInfo {
    FileInfo {
        name,
        full_path,
        icon: "❓",
        color: Color::Red,
        size: 0,
        created: None,
        modified: SystemTime::now(),
        is_dir: false,
        tags: Vec::new(),
    }
}

pub(crate) fn walk_dir_for_regex(
    dir: &Path,
    pattern: &Regex,
    all: bool,
    show_tags: bool,
    recursive: bool,
    tag_manager: &TagManager,
    files: &mut Vec<FileInfo>,
    dirs: &mut Vec<FileInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    let mut builder = WalkDir::new(dir).follow_links(false);
    if !recursive {
        builder = builder.max_depth(1);
    }

    let walker = builder
        .into_iter()
        .filter_entry(|e| all || !is_hidden(&e.path().to_path_buf()));

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        if is_inside_gdb(path) {
            continue;
        }

        let path_str = path.to_string_lossy();

        if pattern.is_match(&path_str) {
            match entry.metadata() {
                Ok(meta) => {
                    let name = path
                        .strip_prefix(&current_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    let full_path = path_str.to_string();
                    let file_info =
                        build_file_info(path, &meta, name, full_path, show_tags, tag_manager);

                    if meta.is_dir() {
                        dirs.push(file_info);
                    } else {
                        files.push(file_info);
                    }
                }
                Err(_) => continue,
            }
        }
    }
    Ok(())
}

pub(crate) fn walk_dir_for_tags(
    dir: &Path,
    all: bool,
    show_tags: bool,
    tag_manager: &TagManager,
    files: &mut Vec<FileInfo>,
    dirs: &mut Vec<FileInfo>,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    let walker = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| all || !is_hidden(&e.path().to_path_buf()));

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        if is_inside_gdb(path) {
            continue;
        }

        let name = path
            .strip_prefix(&current_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        match entry.metadata() {
            Ok(meta) => {
                let full_path = path.to_string_lossy().to_string();
                let file_info =
                    build_file_info(path, &meta, name, full_path, show_tags, tag_manager);

                if meta.is_dir() {
                    dirs.push(file_info);
                } else {
                    files.push(file_info);
                }
            }
            Err(_) => {
                let full_path = path.to_string_lossy().to_string();
                files.push(build_error_file_info(name, full_path));
            }
        }
    }
    Ok(())
}
