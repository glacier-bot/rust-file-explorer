use crate::managers::tag::TagManager;
use crate::messaging;
use colored::*;
use regex::Regex;
use std::env;
use std::path::Path;

#[inline]
fn format_tag_list(tag_manager: &TagManager) -> String {
    let mut output = String::new();
    let all_tags = tag_manager.list_all();
    if all_tags.is_empty() {
        output.push_str(&format!("  {}\n", "No tagged files yet.".bright_black()));
    } else {
        let current_dir = env::current_dir().unwrap_or_default();
        for (path, tags) in all_tags {
            let clean_path = clean_windows_path(path);
            let display_path = match Path::new(&clean_path).strip_prefix(&current_dir) {
                Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                Err(_) => clean_path,
            };
            output.push_str(&format!(
                "  {} -> {}\n",
                display_path.cyan(),
                tags.join(", ").bright_yellow()
            ));
        }
    }
    output
}

#[inline]
fn clean_windows_path(path: &str) -> String {
    if cfg!(windows) && path.starts_with("\\\\?\\") {
        path[4..].to_string()
    } else {
        path.to_string()
    }
}

#[inline]
fn format_tag_usage() -> String {
    let mut output = String::new();
    output.push_str(&format!("\n{} Usage:\n", "💡".bright_green()));
    output.push_str(&format!(
        "  {} <file> <tag1> [tag2...] Add tags to file\n",
        "tag add".cyan().bold()
    ));
    output.push_str(&format!(
        "  {} <file> <tag1> [tag2...] Remove specified tags from file\n",
        "tag remove/rm".cyan().bold()
    ));
    output.push_str(&format!(
        "  {} <file>                Remove all tags from file\n",
        "tag clear".cyan().bold()
    ));
    output.push_str(&format!(
        "  {} <file>                Show all tags for file\n",
        "tag get".cyan().bold()
    ));
    output.push_str(&format!(
        "  {}                        List all tagged files\n",
        "tag list/ls".cyan().bold()
    ));
    output.push_str(&format!(
        "  {} <tag-regex1> [tag-regex2...] Search files by tags\n",
        "tag find/search".cyan().bold()
    ));
    output.push_str(&format!(
        "  {}                        Backup tag data\n",
        "tag backup".cyan().bold()
    ));
    output.push_str(&format!(
        "  {}                       Restore tag data from backup\n",
        "tag restore".cyan().bold()
    ));
    output
}

pub fn cmd_tag(
    tag_manager: &mut TagManager,
    args: &[&str],
) -> Result<(String, String), Box<dyn std::error::Error>> {
    if args.is_empty() {
        let mut output = format!("{}\n\n", "🏷️  Tag List:".bright_yellow().bold());
        output.push_str(&format_tag_list(tag_manager));
        output.push_str(&format_tag_usage());
        output.push_str(&messaging::format_cache_file_note(&tag_manager.config_path));
        return Ok((output, String::new()));
    }

    match args[0].to_lowercase().as_str() {
        "add" => {
            if args.len() < 3 {
                return Err("Usage: tag add <file> <tag1> [tag2...]".into());
            }
            let file_path = args[1];
            let tags = &args[2..];
            tag_manager.add_tags(file_path, tags)?;
            Ok((
                format!(
                    "{} Added tags {} to file {}",
                    "✔️".bright_green(),
                    tags.join(", ").bright_yellow(),
                    file_path.cyan()
                ),
                String::new(),
            ))
        }
        "remove" | "rm" => {
            if args.len() < 3 {
                return Err("Usage: tag remove <file> <tag1> [tag2...]".into());
            }
            let file_path = args[1];
            let tags = &args[2..];
            tag_manager.remove_tags(file_path, tags)?;
            Ok((
                format!(
                    "{} Removed tags {} from file {}",
                    "✔️".bright_green(),
                    tags.join(", ").bright_yellow(),
                    file_path.cyan()
                ),
                String::new(),
            ))
        }
        "clear" => {
            if args.len() < 2 {
                return Err("Usage: tag clear <file>".into());
            }
            let file_path = args[1];
            tag_manager.remove_all_tags(file_path)?;
            Ok((
                format!(
                    "{} Removed all tags from file {}",
                    "✔️".bright_green(),
                    file_path.cyan()
                ),
                String::new(),
            ))
        }
        "get" => {
            if args.len() < 2 {
                return Err("Usage: tag get <file>".into());
            }
            let file_path = args[1];
            let tags = tag_manager.get_tags(file_path);
            if tags.is_empty() {
                Ok((
                    format!("ℹ️  File {} has no tags", file_path.cyan()),
                    String::new(),
                ))
            } else {
                Ok((
                    format!(
                        "🏷️  Tags for {}: {}",
                        file_path.cyan(),
                        tags.join(", ").bright_yellow()
                    ),
                    String::new(),
                ))
            }
        }
        "list" | "ls" => {
            let mut output = format!("{}\n\n", "🏷️ Tag List:".bright_yellow().bold());
            output.push_str(&format_tag_list(tag_manager));
            Ok((output, String::new()))
        }
        "backup" => {
            tag_manager.backup()?;
            Ok((
                "✔️ Tag data backed up successfully"
                    .bright_green()
                    .to_string(),
                String::new(),
            ))
        }
        "find" | "search" => {
            if args.len() < 2 {
                return Err("Usage: tag find <tag-regex1> [tag-regex2...]".into());
            }

            let mut tag_patterns = Vec::new();
            for pattern_str in &args[1..] {
                match Regex::new(pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("Invalid tag regex: {}", e).into()),
                }
            }

            let results = tag_manager.find_files_by_tags(&tag_patterns);
            let mut output = format!(
                "{} Found {} files matching tags:\n\n",
                "🔍".bright_yellow().bold(),
                results.len()
            );

            if results.is_empty() {
                output.push_str(&format!(
                    "  {}\n",
                    "No matching files found.".bright_black()
                ));
            } else {
                let current_dir = env::current_dir()?;
                for (path, tags) in results {
                    let display_path = match Path::new(&path).strip_prefix(&current_dir) {
                        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                        Err(_) => path,
                    };
                    output.push_str(&format!(
                        "  {} -> {}\n",
                        display_path.cyan(),
                        tags.join(", ").bright_yellow()
                    ));
                }
            }

            Ok((output, String::new()))
        }
        "restore" => {
            tag_manager.restore()?;
            Ok((
                "✔️ Tag data restored from backup successfully"
                    .bright_green()
                    .to_string(),
                String::new(),
            ))
        }
        _ => Err(format!("Unknown tag subcommand: {}", args[0]).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_bare_tag_shows_cache_file_path() {
        let config = tempdir().unwrap();
        let mut tag_manager = TagManager::with_config_dir(config.path().to_path_buf()).unwrap();

        let (display, _) = cmd_tag(&mut tag_manager, &[]).unwrap();
        let expected = config.path().join("tags.json").display().to_string();
        assert!(display.contains("Cache file:"), "{}", display);
        assert!(display.contains(&expected), "{}", display);
    }

    #[test]
    fn test_tag_list_subcommand_has_no_cache_path_line() {
        let config = tempdir().unwrap();
        let mut tag_manager = TagManager::with_config_dir(config.path().to_path_buf()).unwrap();

        let (display, _) = cmd_tag(&mut tag_manager, &["list"]).unwrap();
        assert!(!display.contains("Cache file:"), "{}", display);
    }
}
