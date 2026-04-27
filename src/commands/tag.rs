use colored::*;
use regex::Regex;
use std::env;
use std::path::Path;
use crate::managers::tag::TagManager;

pub fn cmd_tag(tag_manager: &mut TagManager, args: &[&str]) -> Result<(String, String), Box<dyn std::error::Error>> {
    if args.is_empty() {
        let mut output = format!("{}\n\n", "🏷️  标签管理命令帮助:".bright_yellow().bold());
        output.push_str(&format!("  {} <文件> <标签1> [标签2...] 添加标签到文件\n", "tag add".cyan().bold()));
        output.push_str(&format!("  {} <文件> <标签1> [标签2...] 移除文件的指定标签\n", "tag remove/rm".cyan().bold()));
        output.push_str(&format!("  {} <文件>                移除文件的所有标签\n", "tag clear".cyan().bold()));
        output.push_str(&format!("  {} <文件>                查看文件的所有标签\n", "tag get".cyan().bold()));
        output.push_str(&format!("  {}                        列出所有带标签的文件\n", "tag list/ls".cyan().bold()));
        output.push_str(&format!("  {} <标签正则1> [标签正则2...] 全局搜索所有匹配标签的文件\n", "tag find/search".cyan().bold()));
        output.push_str(&format!("  {}                        备份标签数据\n", "tag backup".cyan().bold()));
        output.push_str(&format!("  {}                        从备份恢复标签数据\n", "tag restore".cyan().bold()));
        return Ok((output, String::new()));
    }
    
    match args[0].to_lowercase().as_str() {
        "add" => {
            if args.len() < 3 {
                return Err("用法: tag add <文件> <标签1> [标签2...]".into());
            }
            let file_path = args[1];
            let tags = &args[2..];
            tag_manager.add_tags(file_path, tags)?;
            Ok((format!("✔️  已为文件 {} 添加标签: {}", file_path.cyan(), tags.join(", ").bright_yellow()), String::new()))
        }
        "remove" | "rm" => {
            if args.len() < 3 {
                return Err("用法: tag remove <文件> <标签1> [标签2...]".into());
            }
            let file_path = args[1];
            let tags = &args[2..];
            tag_manager.remove_tags(file_path, tags)?;
            Ok((format!("✔️  已为文件 {} 移除标签: {}", file_path.cyan(), tags.join(", ").bright_yellow()), String::new()))
        }
        "clear" => {
            if args.len() < 2 {
                return Err("用法: tag clear <文件>".into());
            }
            let file_path = args[1];
            tag_manager.remove_all_tags(file_path)?;
            Ok((format!("✔️  已为文件 {} 移除所有标签", file_path.cyan()), String::new()))
        }
        "get" => {
            if args.len() < 2 {
                return Err("用法: tag get <文件>".into());
            }
            let file_path = args[1];
            let tags = tag_manager.get_tags(file_path);
            if tags.is_empty() {
                Ok((format!("ℹ️  文件 {} 没有任何标签", file_path.cyan()), String::new()))
            } else {
                Ok((format!("🏷️  文件 {} 的标签: {}", file_path.cyan(), tags.join(", ").bright_yellow()), String::new()))
            }
        }
        "list" | "ls" => {
            let mut output = format!("{}\n\n", "🏷️  所有带标签的文件:".bright_yellow().bold());
            let all_tags = tag_manager.list_all();
            if all_tags.is_empty() {
                output.push_str(&format!("  {}\n", "没有任何标签记录".bright_black()));
            } else {
                for (path, tags) in all_tags {
                    let mut clean_path = path.clone();
                    if cfg!(windows) && clean_path.starts_with("\\\\?\\") {
                        clean_path = clean_path[4..].to_string();
                    }
                    
                    let display_path = match Path::new(&clean_path).strip_prefix(env::current_dir()?) {
                        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                        Err(_) => clean_path
                    };
                    output.push_str(&format!("  {} -> {}\n", display_path.cyan(), tags.join(", ").bright_yellow()));
                }
            }
            Ok((output, String::new()))
        }
        "backup" => {
            tag_manager.backup()?;
            Ok(("✔️  标签数据已备份成功".bright_green().to_string(), String::new()))
        }
        "find" | "search" => {
            if args.len() < 2 {
                return Err("用法: tag find <标签正则1> [标签正则2...]".into());
            }
            
            let mut tag_patterns = Vec::new();
            for pattern_str in &args[1..] {
                match Regex::new(pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("标签正则表达式无效: {}", e).into()),
                }
            }
            
            let results = tag_manager.find_files_by_tags(&tag_patterns);
            let mut output = format!("{} 匹配标签的文件共{}个:\n\n", "🔍".bright_yellow().bold(), results.len());
            
            if results.is_empty() {
                output.push_str(&format!("  {}\n", "没有找到匹配的文件".bright_black()));
            } else {
                let current_dir = env::current_dir()?;
                for (path, tags) in results {
                    let display_path = match Path::new(&path).strip_prefix(&current_dir) {
                        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                        Err(_) => path
                    };
                    output.push_str(&format!("  {} -> {}\n", display_path.cyan(), tags.join(", ").bright_yellow()));
                }
            }
            
            Ok((output, String::new()))
        }
        "restore" => {
            tag_manager.restore()?;
            Ok(("✔️  标签数据已从备份恢复成功".bright_green().to_string(), String::new()))
        }
        _ => {
            Err(format!("未知的tag子命令: {}", args[0]).into())
        }
    }
}