use colored::{Color, Colorize};
use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use unicode_width::UnicodeWidthStr;
use walkdir::WalkDir;

use crate::managers::tag::TagManager;
use crate::models::FileInfo;
use crate::utils::format::{center_text, format_size, format_time_absolute, pad_to_width, truncate_string};
use crate::utils::terminal::{calculate_column_widths, get_terminal_width, make_separator};
use crate::utils::path::is_hidden;
use crate::utils::moe::is_moe;

/// 文件扩展名到图标和颜色的映射
const FILE_ICONS: &[(&[&str], &str, Color)] = &[
    (&["rs"], "🦀", Color::BrightRed),
    (&["toml", "json", "yaml", "yml"], "📋", Color::BrightYellow),
    (&["md", "txt"], "📝", Color::White),
    (&["gitignore", "git"], "🔀", Color::BrightMagenta),
    (&["exe", "bin"], "⚡", Color::BrightGreen),
    (&["jpg", "jpeg", "png", "gif", "svg"], "📷", Color::Magenta),
    (&["mp3", "wav", "flac"], "🎵", Color::BrightMagenta),
    (&["mp4", "avi", "mkv"], "🎬", Color::Red),
    (&["zip", "tar", "gz", "7z", "rar"], "📦", Color::BrightRed),
    (&["pdf"], "📕", Color::Red),
    (&["doc", "docx"], "📘", Color::BrightBlue),
    (&["xls", "xlsx"], "📗", Color::BrightGreen),
    (&["ppt", "pptx"], "📙", Color::BrightYellow),
    (&["html", "css", "js", "ts"], "🌐", Color::BrightCyan),
    (&["py"], "🐍", Color::BrightYellow),
    (&["go"], "🐹", Color::BrightCyan),
    (&["java"], "☕", Color::BrightRed),
    (&["c", "cpp", "h", "hpp"], "🔧", Color::BrightBlue),
    (&["sh", "bat", "ps1"], "💻", Color::BrightGreen),
    (&["lock"], "🔒", Color::BrightYellow),
    (&["log"], "📜", Color::BrightBlack),
];

pub fn get_file_icon_and_color(path: &Path, metadata: &std::fs::Metadata) -> (&'static str, Color) {
    if metadata.is_dir() {
        return ("📁", Color::BrightBlue);
    }
    if metadata.is_symlink() {
        return ("🔗", Color::Cyan);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    for &(exts, icon, color) in FILE_ICONS {
        if exts.contains(&ext.as_str()) {
            return (icon, color);
        }
    }

    ("📄", Color::White)
}

#[inline]
fn is_inside_gdb(path: &Path) -> bool {
    path.ancestors().skip(1).any(|a| {
        a.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".gdb"))
    })
}

#[inline]
fn build_file_info(
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
fn build_error_file_info(name: String, full_path: String) -> FileInfo {
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

fn walk_dir_for_regex(
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

    let walker = builder.into_iter().filter_entry(|e| {
        all || !is_hidden(&e.path().to_path_buf())
    });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();

        if is_inside_gdb(path) {
            continue;
        }

        let path_str = path.to_string_lossy();

        if pattern.is_match(&path_str) {
            match entry.metadata() {
                Ok(meta) => {
                    let name = path.strip_prefix(&current_dir)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    let full_path = path_str.to_string();
                    let file_info = build_file_info(path, &meta, name, full_path, show_tags, tag_manager);

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

fn walk_dir_for_tags(
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

        let name = path.strip_prefix(&current_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        match entry.metadata() {
            Ok(meta) => {
                let full_path = path.to_string_lossy().to_string();
                let file_info = build_file_info(path, &meta, name, full_path, show_tags, tag_manager);

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
        }.map_err(|e| format!("Invalid regular expression: {}", e))?;

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

        walk_dir_for_regex(&search_dir, &re_pattern, all, show_tags, recursive, tag_manager, &mut files, &mut dirs)?;
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
                        let file_info = build_file_info(&path, &meta, name, full_path, show_tags, tag_manager);

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
        render_long_format(&mut output, &all_items, show_tags);
    } else {
        render_short_format(&mut output, &all_items, show_tags);
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

    Ok((output, raw_path, all_items))
}

fn render_long_format(output: &mut String, all_items: &[FileInfo], show_tags: bool) {
    let term_width = get_terminal_width();
    let (name_width, created_width, modified_width, size_width, tags_width) = 
        calculate_column_widths(term_width, show_tags);
    let truncate_name_width = name_width.saturating_sub(4);

    if show_tags {
        let widths = [3, name_width, created_width, modified_width, size_width, tags_width];
        let separator = make_separator(&widths).bright_black();

        output.push_str(&format!("{}\n", separator));
        output.push_str(&format!(
            "| {:^3} | {} | {} | {} | {} | {} |\n",
            "#".bright_white().bold(),
            center_text("Name", name_width).bright_white().bold(),
            center_text("Created Date", created_width).bright_white().bold(),
            center_text("Modified Date", modified_width).bright_white().bold(),
            center_text("Size", size_width).bright_white().bold(),
            center_text("Tags", tags_width).bright_white().bold(),
        ));
        output.push_str(&format!("{}\n", separator));

        for (idx, item) in all_items.iter().enumerate() {
            let line_num = idx + 1;
            let created_str = item.created.map_or_else(|| "N/A".to_string(), format_time_absolute);
            let modified_str = format_time_absolute(item.modified);
            let tags_str = if item.tags.is_empty() {
                String::new()
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
                "| {:3} | {} | {} | {} | {} | {} |\n",
                line_num,
                padded_name.color(item.color).bold(),
                pad_to_width(&truncate_string(&created_str, created_width), created_width).bright_cyan(),
                pad_to_width(&truncate_string(&modified_str, modified_width), modified_width).bright_magenta(),
                pad_to_width(&format_size(item.size), size_width).bright_yellow().bold(),
                padded_tags.bright_yellow()
            ));
        }

        output.push_str(&format!("{}\n", separator));
    } else {
        let widths = [3, name_width, created_width, modified_width, size_width];
        let separator = make_separator(&widths).bright_black();

        output.push_str(&format!("{}\n", separator));
        output.push_str(&format!(
            "| {:^3} | {} | {} | {} | {} |\n",
            "#".bright_white().bold(),
            center_text("Name", name_width).bright_white().bold(),
            center_text("Created Date", created_width).bright_white().bold(),
            center_text("Modified Date", modified_width).bright_white().bold(),
            center_text("Size", size_width).bright_white().bold(),
        ));
        output.push_str(&format!("{}\n", separator));

        for (idx, item) in all_items.iter().enumerate() {
            let line_num = idx + 1;
            let created_str = item.created.map_or_else(|| "N/A".to_string(), format_time_absolute);
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
                "| {:3} | {} | {} | {} | {} |\n",
                line_num,
                padded_name.color(item.color).bold(),
                pad_to_width(&truncate_string(&created_str, created_width), created_width).bright_cyan(),
                pad_to_width(&truncate_string(&modified_str, modified_width), modified_width).bright_magenta(),
                pad_to_width(&format_size(item.size), size_width).bright_yellow().bold()
            ));
        }

        output.push_str(&format!("{}\n", separator));
    }
}

fn render_short_format(output: &mut String, all_items: &[FileInfo], show_tags: bool) {
    for (idx, item) in all_items.iter().enumerate() {
        let line_num = idx + 1;
        let display_name = truncate_string(&item.name, 50);
        if show_tags && !item.tags.is_empty() {
            let tags_str = format!(" [{}]", item.tags.join(", "));
            output.push_str(&format!("{:3}. {} {}{}\n", line_num, item.icon, display_name.color(item.color).bold(), tags_str.bright_yellow()));
        } else {
            output.push_str(&format!("{:3}. {} {}\n", line_num, item.icon, display_name.color(item.color).bold()));
        }
    }
}
