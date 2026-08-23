use colored::Colorize;
use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::ls_walk::{
    build_error_file_info, build_file_info, walk_dir_for_regex, walk_dir_for_tags,
};
use crate::managers::tag::TagManager;
use crate::models::FileInfo;
use crate::utils::moe::is_moe;
use crate::utils::path::is_hidden;

#[allow(clippy::too_many_arguments)]
pub fn cmd_ls(
    all: bool,
    long: bool,
    re: bool,
    re_insensitive: bool,
    show_tags: bool,
    recursive: bool,
    path: Option<&str>,
    tag_manager: &TagManager,
    tag_patterns: &[Regex],
) -> Result<(String, String, Vec<FileInfo>), Box<dyn std::error::Error>> {
    let mut output = String::new();
    let mut files: Vec<FileInfo> = Vec::new();
    let mut dirs: Vec<FileInfo> = Vec::new();

    if re {
        let pattern = path.ok_or("Regex pattern required when using --re flag")?;

        let re_pattern = if re_insensitive {
            Regex::new(&format!("(?i){}", pattern))
        } else {
            Regex::new(pattern)
        }
        .map_err(|e| format!("Invalid regular expression: {}", e))?;

        let search_dir = if pattern.starts_with('/') || (cfg!(windows) && pattern.contains(':')) {
            PathBuf::from("/")
        } else {
            env::current_dir()?
        };

        if is_moe() {
            output.push_str(&format!(
                "{} {}\n\n",
                "💫🔍 Regex Search~:".truecolor(255, 160, 122).bold(),
                pattern.truecolor(255, 182, 193)
            ));
        } else {
            output.push_str(&format!(
                "{} {}\n\n",
                "🔍 Regex Search:".bright_yellow().bold(),
                pattern.bright_cyan()
            ));
        }

        walk_dir_for_regex(
            &search_dir,
            &re_pattern,
            all,
            show_tags,
            recursive,
            tag_manager,
            &mut files,
            &mut dirs,
        )?;
    } else {
        let target = match path {
            Some(p) => PathBuf::from(p),
            None => env::current_dir()?,
        };

        if !target.exists() {
            return Err(format!("Path does not exist: {}", target.display()).into());
        }

        if is_moe() {
            output.push_str(&format!(
                "{} {}\n\n",
                "🌸📂 Directory~:".truecolor(255, 160, 122).bold(),
                target.display().to_string().truecolor(255, 182, 193)
            ));
        } else {
            output.push_str(&format!(
                "{} {}\n\n",
                "📂 Directory:".bright_yellow().bold(),
                target.display().to_string().bright_cyan()
            ));
        }

        if !tag_patterns.is_empty() && recursive {
            walk_dir_for_tags(&target, all, show_tags, tag_manager, &mut files, &mut dirs)?;
        } else {
            let entries = fs::read_dir(target)?;

            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                if !all && is_hidden(&path) {
                    continue;
                }

                let name = entry.file_name().to_string_lossy().to_string();

                match entry.metadata() {
                    Ok(meta) => {
                        let full_path = path.to_string_lossy().to_string();
                        let file_info =
                            build_file_info(&path, &meta, name, full_path, show_tags, tag_manager);

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
        }
    }

    dirs.sort_by_key(|a| a.name.to_lowercase());
    files.sort_by_key(|a| a.name.to_lowercase());

    let mut all_items = Vec::with_capacity(dirs.len() + files.len());
    all_items.extend(dirs);
    all_items.extend(files);

    if !tag_patterns.is_empty() {
        let current_dir = env::current_dir().unwrap_or_default();
        all_items.retain(|item| {
            let full_path = if recursive {
                item.name.clone()
            } else {
                match path {
                    Some(p) => Path::new(p).join(&item.name).to_string_lossy().to_string(),
                    None => current_dir.join(&item.name).to_string_lossy().to_string(),
                }
            };
            tag_manager.file_matches_tags(&full_path, tag_patterns)
        });
    }

    if long {
        crate::commands::render::render_long_format(&mut output, &all_items, show_tags);
    } else {
        crate::commands::render::render_short_format(&mut output, &all_items, show_tags);
    }

    output.push('\n');
    let total = all_items.len();
    let dir_count = all_items.iter().filter(|f| f.is_dir).count();
    let file_count = total - dir_count;

    if is_moe() {
        output.push_str(&format!(
            "{} {} directories, {} files~ 💕\n\n",
            "✨📊".truecolor(255, 105, 180),
            dir_count.to_string().truecolor(255, 182, 193),
            file_count.to_string().truecolor(255, 192, 203)
        ));
    } else {
        output.push_str(&format!(
            "{} {} directories, {} files\n\n",
            "📊".bright_green(),
            dir_count.to_string().bright_blue(),
            file_count.to_string().bright_cyan()
        ));
    }

    let raw_path = if re {
        let search_dir = match path {
            Some(p) if p.starts_with('/') || (cfg!(windows) && p.contains(':')) => {
                PathBuf::from("/")
            }
            _ => env::current_dir()?,
        };
        search_dir.display().to_string()
    } else {
        match path {
            Some(p) => PathBuf::from(p).display().to_string(),
            None => env::current_dir()?.display().to_string(),
        }
    };

    Ok((output, raw_path, all_items))
}
