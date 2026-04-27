use colored::{Color, Colorize};
use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use unicode_width::UnicodeWidthStr;

use crate::managers::tag::TagManager;
use crate::models::FileInfo;
use crate::utils::format::{center_text, format_size, format_time_absolute, pad_to_width, truncate_string};
use crate::utils::terminal::{calculate_column_widths, get_terminal_width, make_separator};
use crate::utils::path::is_hidden;

pub fn get_file_icon_and_color(path: &Path, metadata: &std::fs::Metadata) -> (&'static str, Color) {
    if metadata.is_dir() {
        ("📁", Color::BrightBlue)
    } else if metadata.is_symlink() {
        ("🔗", Color::Cyan)
    } else {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => ("🦀", Color::BrightRed),
            "toml" | "json" | "yaml" | "yml" => ("📋", Color::BrightYellow),
            "md" | "txt" => ("📝", Color::White),
            "gitignore" | "git" => ("🔀", Color::BrightMagenta),
            "exe" | "bin" => ("⚡", Color::BrightGreen),
            "jpg" | "jpeg" | "png" | "gif" | "svg" => ("📷", Color::Magenta),
            "mp3" | "wav" | "flac" => ("🎵", Color::BrightMagenta),
            "mp4" | "avi" | "mkv" => ("🎬", Color::Red),
            "zip" | "tar" | "gz" | "7z" | "rar" => ("📦", Color::BrightRed),
            "pdf" => ("📕", Color::Red),
            "doc" | "docx" => ("📘", Color::BrightBlue),
            "xls" | "xlsx" => ("📗", Color::BrightGreen),
            "ppt" | "pptx" => ("📙", Color::BrightYellow),
            "html" | "css" | "js" | "ts" => ("🌐", Color::BrightCyan),
            "py" => ("🐍", Color::BrightYellow),
            "go" => ("🐹", Color::BrightCyan),
            "java" => ("☕", Color::BrightRed),
            "c" | "cpp" | "h" | "hpp" => ("🔧", Color::BrightBlue),
            "sh" | "bat" | "ps1" => ("💻", Color::BrightGreen),
            "lock" => ("🔒", Color::BrightYellow),
            "log" => ("📜", Color::BrightBlack),
            _ => ("📄", Color::White),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_ls(all: bool, long: bool, re: bool, re_insensitive: bool, show_tags: bool, recursive: bool, path: Option<&str>, tag_manager: &TagManager, tag_patterns: &[Regex]) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut output = String::new();
    let mut files: Vec<FileInfo> = Vec::new();
    let mut dirs: Vec<FileInfo> = Vec::new();

    if re {
        let pattern = path.ok_or("Regex pattern required when using --re flag")?;
        
        let re_pattern = if re_insensitive {
            Regex::new(&format!("(?i){}", pattern))
        } else {
            Regex::new(pattern)
        }.map_err(|e| format!("Invalid regular expression: {}", e))?;

        let search_dir = if pattern.starts_with('/') || (cfg!(windows) && pattern.contains(':')) {
            PathBuf::from("/")
        } else {
            env::current_dir()?
        };

        output.push_str(&format!(
            "{} {}\n\n",
            "🔍 Regex Search:".bright_yellow().bold(),
            pattern.bright_cyan()
        ));

        fn walk_dir(dir: &Path, pattern: &Regex, all: bool, show_tags: bool, recursive: bool, tag_manager: &TagManager, files: &mut Vec<FileInfo>, dirs: &mut Vec<FileInfo>) -> Result<(), Box<dyn std::error::Error>> {
            for entry in fs::read_dir(dir)?.filter_map(|e| e.ok()) {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                if !all && is_hidden(&path) {
                    continue;
                }

                if pattern.is_match(&path_str) {
                    match entry.metadata() {
                        Ok(meta) => {
                            let (icon, color) = get_file_icon_and_color(&path, &meta);
                            let created = meta.created().ok();
                            let modified = meta.modified().unwrap_or_else(|_| SystemTime::now());
                            let name = path.strip_prefix(env::current_dir()?)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string();

                            let tags = if show_tags {
                                tag_manager.get_tags(path.to_str().unwrap_or(""))
                            } else {
                                Vec::new()
                            };
                            
                            let file_info = FileInfo {
                                name,
                                icon,
                                color,
                                size: meta.len(),
                                created,
                                modified,
                                is_dir: meta.is_dir(),
                                tags,
                            };

                            if meta.is_dir() {
                                dirs.push(file_info);
                            } else {
                                files.push(file_info);
                            }
                        }
                        Err(_) => continue,
                    }
                }

                if path.is_dir() && recursive && walk_dir(&path, pattern, all, show_tags, recursive, tag_manager, files, dirs).is_err() {
                    continue;
                }
            }
            Ok(())
        }

        walk_dir(&search_dir, &re_pattern, all, show_tags, recursive, tag_manager, &mut files, &mut dirs)?
    } else {
        let target = match path {
            Some(p) => PathBuf::from(p),
            None => env::current_dir()?,
        };

        if !target.exists() {
            return Err(format!("Path does not exist: {}", target.display()).into());
        }

        output.push_str(&format!(
            "{} {}\n\n",
            "📂 Directory:".bright_yellow().bold(),
            target.display().to_string().bright_cyan()
        ));

        let entries = fs::read_dir(target)?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if !all && is_hidden(&path) {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();

            match entry.metadata() {
                Ok(meta) => {
                    let (icon, color) = get_file_icon_and_color(&path, &meta);
                    let created = meta.created().ok();
                    let modified = meta.modified().unwrap_or_else(|_| SystemTime::now());

                    let tags = if show_tags {
                        tag_manager.get_tags(path.to_str().unwrap_or(""))
                    } else {
                        Vec::new()
                    };
                    
                    let file_info = FileInfo {
                        name,
                        icon,
                        color,
                        size: meta.len(),
                        created,
                        modified,
                        is_dir: meta.is_dir(),
                        tags,
                    };

                    if meta.is_dir() {
                        dirs.push(file_info);
                    } else {
                        files.push(file_info);
                    }
                }
                Err(_) => {
                    let file_info = FileInfo {
                        name,
                        icon: "❓",
                        color: Color::Red,
                        size: 0,
                        created: None,
                        modified: SystemTime::now(),
                        is_dir: false,
                        tags: Vec::new(),
                    };
                    files.push(file_info);
                }
            }
        }
    }

    dirs.sort_by_key(|a| a.name.to_lowercase());
    files.sort_by_key(|a| a.name.to_lowercase());

    let mut all_items = Vec::new();
    all_items.extend(dirs);
    all_items.extend(files);
    
    if !tag_patterns.is_empty() {
        all_items.retain(|item| {
            let full_path = match &path {
                Some(p) => Path::new(p).join(&item.name),
                None => env::current_dir().unwrap_or_default().join(&item.name),
            };
            tag_manager.file_matches_tags(full_path.to_str().unwrap_or_default(), tag_patterns)
        });
    }

    if long {
        let term_width = get_terminal_width();
        let (name_width, created_width, modified_width, size_width, tags_width) = 
            calculate_column_widths(term_width, show_tags);
        let truncate_name_width = name_width.saturating_sub(4);

        if show_tags {
            let widths = [name_width, created_width, modified_width, size_width, tags_width];
            let separator = make_separator(&widths).bright_black();

            output.push_str(&format!("{}\n", separator));
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                center_text("Name", name_width).bright_white().bold(),
                center_text("Created Date", created_width).bright_white().bold(),
                center_text("Modified Date", modified_width).bright_white().bold(),
                center_text("Size", size_width).bright_white().bold(),
                center_text("Tags", tags_width).bright_white().bold(),
            ));
            output.push_str(&format!("{}\n", separator));

            for item in &all_items {
                let created_str = item
                    .created
                    .map_or("N/A".to_string(), format_time_absolute);
                let modified_str = format_time_absolute(item.modified);
                let tags_str = if item.tags.is_empty() {
                    "".to_string()
                } else {
                    item.tags.join(", ")
                };

                let display_name = truncate_string(&item.name, truncate_name_width);
                let display_text = format!("{}  {}", item.icon, display_name);
                let display_text_width = display_text.width();
                let padding = if display_text_width < name_width {
                    " ".repeat(name_width - display_text_width)
                } else {
                    String::new()
                };
                let padded_name = format!("{}{}", display_text, padding);
                let padded_tags = pad_to_width(&truncate_string(&tags_str, tags_width), tags_width);

                output.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    padded_name.color(item.color).bold(),
                    pad_to_width(&truncate_string(&created_str, created_width), created_width).bright_cyan(),
                    pad_to_width(&truncate_string(&modified_str, modified_width), modified_width).bright_magenta(),
                    pad_to_width(&format_size(item.size), size_width).bright_yellow().bold(),
                    padded_tags.bright_yellow()
                ));
            }

            output.push_str(&format!("{}\n", separator));
        } else {
            let widths = [name_width, created_width, modified_width, size_width];
            let separator = make_separator(&widths).bright_black();

            output.push_str(&format!("{}\n", separator));
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                center_text("Name", name_width).bright_white().bold(),
                center_text("Created Date", created_width).bright_white().bold(),
                center_text("Modified Date", modified_width).bright_white().bold(),
                center_text("Size", size_width).bright_white().bold(),
            ));
            output.push_str(&format!("{}\n", separator));

            for item in &all_items {
                let created_str = item
                    .created
                    .map_or("N/A".to_string(), format_time_absolute);
                let modified_str = format_time_absolute(item.modified);

                let display_name = truncate_string(&item.name, truncate_name_width);
                let display_text = format!("{}  {}", item.icon, display_name);
                let display_text_width = display_text.width();
                let padding = if display_text_width < name_width {
                    " ".repeat(name_width - display_text_width)
                } else {
                    String::new()
                };
                let padded_name = format!("{}{}", display_text, padding);

                output.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    padded_name.color(item.color).bold(),
                    pad_to_width(&truncate_string(&created_str, created_width), created_width).bright_cyan(),
                    pad_to_width(&truncate_string(&modified_str, modified_width), modified_width).bright_magenta(),
                    pad_to_width(&format_size(item.size), size_width).bright_yellow().bold()
                ));
            }

            output.push_str(&format!("{}\n", separator));
        }
    } else {
        for item in &all_items {
            let display_name = truncate_string(&item.name, 50);
            if show_tags && !item.tags.is_empty() {
                let tags_str = format!(" [{}]", item.tags.join(", "));
                output.push_str(&format!("  {} {}{}\n", item.icon, display_name.color(item.color).bold(), tags_str.bright_yellow()));
            } else {
                output.push_str(&format!("  {} {}\n", item.icon, display_name.color(item.color).bold()));
            }
        }
    }

    output.push('\n');
    let total = all_items.len();
    let dir_count = all_items.iter().filter(|f| f.is_dir).count();
    let file_count = total - dir_count;

    output.push_str(&format!(
        "{} {} directories, {} files\n\n",
        "📊".bright_green(),
        dir_count.to_string().bright_blue(),
        file_count.to_string().bright_cyan()
    ));
    
    let raw_path = if re {
        let search_dir = match path {
            Some(p) if p.starts_with('/') || (cfg!(windows) && p.contains(':')) => PathBuf::from("/"),
            _ => env::current_dir()?
        };
        search_dir.display().to_string()
    } else {
        match path {
            Some(p) => PathBuf::from(p).display().to_string(),
            None => env::current_dir()?.display().to_string()
        }
    };

    Ok((output, raw_path))
}