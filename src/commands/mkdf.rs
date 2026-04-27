use colored::*;
use std::fs::{self, File};
use std::path::PathBuf;

pub fn cmd_mkdf(args: &[&str]) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut create_file = false;
    let mut create_dir = false;
    let mut parents = false;
    let mut path: Option<String> = None;
    let mut show_help = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-f" | "--file" => {
                create_file = true;
                create_dir = false;
            }
            "-d" | "--directory" => {
                create_dir = true;
                create_file = false;
            }
            "-p" | "--parents" => {
                parents = true;
            }
            "-h" | "--help" => {
                show_help = true;
            }
            p => {
                if path.is_none() {
                    path = Some(p.to_string());
                } else {
                    return Err("Too many arguments. Only one path can be specified.".into());
                }
            }
        }
        i += 1;
    }
    
    if show_help {
        let mut output = format!("{}\n\n", "📁 mkdf Command Help:".bright_yellow().bold());
        output.push_str(&format!("  {} Create a file\n", "mkdf -f/--file <path>".cyan().bold()));
        output.push_str(&format!("  {} Create a directory\n", "mkdf -d/--directory <path>".cyan().bold()));
        output.push_str(&format!("  {} Create parent directories if they don't exist\n", "mkdf -p/--parents".cyan().bold()));
        output.push_str(&format!("  {} Show this help\n\n", "mkdf -h/--help".cyan().bold()));
        output.push_str(&format!("{}\n", "Examples:".bright_green().bold()));
        output.push_str(&format!("  {} Create a file named 'test.txt'\n", "mkdf -f test.txt".cyan()));
        output.push_str(&format!("  {} Create a directory named 'new_folder'\n", "mkdf -d new_folder".cyan()));
        output.push_str(&format!("  {} Create a file with parent directories\n", "mkdf -f -p path/to/file.txt".cyan()));
        output.push_str(&format!("  {} Create nested directories\n", "mkdf -d -p parent/child/grandchild".cyan()));
        return Ok((output, String::new()));
    }
    
    let path = path.ok_or("Usage: mkdf [-f|--file|-d|--directory] [-p|--parents] <path>")?;
    
    if !create_file && !create_dir {
        return Err("Please specify whether to create a file (-f/--file) or directory (-d/--directory)".into());
    }
    
    let target_path = PathBuf::from(&path);
    
    if target_path.exists() {
        return Err(format!("Path already exists: {}", target_path.display()).into());
    }
    
    if create_file {
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        File::create(&target_path)?;
        Ok((format!("{} Created file: {}", "✔".bright_green(), target_path.display().to_string().cyan()), target_path.display().to_string()))
    } else {
        if parents {
            fs::create_dir_all(&target_path)?;
            Ok((format!("{} Created directory (with parents): {}", "✔".bright_green(), target_path.display().to_string().cyan()), target_path.display().to_string()))
        } else {
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    return Err(format!("Parent directory does not exist. Use -p/--parents to create it: {}", parent.display()).into());
                }
            }
            
            fs::create_dir(&target_path)?;
            Ok((format!("{} Created directory: {}", "✔".bright_green(), target_path.display().to_string().cyan()), target_path.display().to_string()))
        }
    }
}