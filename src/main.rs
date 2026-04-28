use colored::Colorize;
use rustyline::{Cmd, KeyCode, KeyEvent, Movement};
use std::env;

mod utils;
mod cache;
mod models;
mod managers;
mod commands;
mod helper;

use crate::utils::split_command_args;

use crate::commands::*;
use crate::helper::RfeHelper;
use crate::managers::{alias::AliasManager, tag::TagManager};
use rustyline::completion::FilenameCompleter;

fn print_welcome() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".bright_green()
    );
    println!(
        "{}",
        "║           Rust File Explorer v0.3.2                          ║".bright_green()
    );
    println!(
        "{}",
        "║           A cross-platform CLI file browser                  ║".bright_green()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".bright_green()
    );
    println!();
    println!("{}", "Commands:".bright_yellow());
    println!("  {}  - List directory contents (supports --re regex search, -t tag filter)", "ls".cyan());
    println!("  {}  - Print working directory", "pwd".cyan());
    println!("  {}  - Copy current directory path to clipboard", "cppwd".cyan());
    println!("  {}  - Copy file absolute path to clipboard", "cpf <file>".cyan());
    println!("  {}  - Change directory", "cd <path>".cyan());
    println!("  {}  - Open file with default application / Open directory in file explorer", "open <path>".cyan());
    println!("  {}  - Move/copy file or folder (use --cp to copy)", "mv <source> <dest> [--cp]".cyan());
    println!("  {}  - Create file or directory (-f for file, -d for directory)", "mkdf".cyan());
    println!("  {}  - Manage path aliases (add/remove/list)", "alias".cyan());
    println!("  {}  - Manage file tags (add/remove/find/list/backup)", "tag".cyan());
    println!("  {}  - Exit the program", "exit".cyan());
    println!("  {} - Clear the screen", "clear".cyan());
    println!("  {}  - Show this help", "help".cyan());
    println!();
    println!("{}", "Keyboard shortcuts:".bright_yellow());
    println!("  {} - Clear current input line in REPL mode", "ESC".cyan());
    println!();
}

fn get_prompt_string() -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    let dir_str = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");

    format!("rfe {} >", dir_str)
}

fn execute_single_command(input: &str, input_data: &str, alias_manager: &mut AliasManager, tag_manager: &mut TagManager) -> Result<(bool, String, String), Box<dyn std::error::Error>> {
    let parts: Vec<String> = split_command_args(input);

    if parts.is_empty() {
        return Ok((false, String::new(), String::new()));
    }

    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "pwd" => {
            let (display, raw) = pwd::cmd_pwd()?;
            Ok((false, display, raw))
        }
        "cppwd" => {
            let (display, raw) = clipboard::cmd_cppwd()?;
            Ok((false, display, raw))
        }
        "cpf" => {
            let path = if let Some(p) = parts.get(1) {
                p.clone()
            } else if !input_data.is_empty() {
                input_data.to_string()
            } else {
                return Err("Usage: cpf <file>".into());
            };
            let resolved_path = alias_manager.resolve_path(&path);
            let (display, raw) = clipboard::cmd_cpf(&resolved_path)?;
            Ok((false, display, raw))
        }
        "cd" => {
            let path = if parts.len() > 1 {
                Some(alias_manager.resolve_path(&parts[1]))
            } else if !input_data.is_empty() {
                Some(input_data.to_string())
            } else {
                None
            };
            let (display, raw) = cd::cmd_cd(path.as_deref())?;
            Ok((false, display, raw))
        }
        "ls" => {
            let mut all = false;
            let mut long = false;
            let mut re = false;
            let mut re_insensitive = false;
            let mut show_tags = false;
            let mut recursive = false;
            let mut path: Option<String> = None;
            let mut tag_pattern_strs: Vec<String> = Vec::new();
            
            let mut i = 1;
            while i < parts.len() {
                match parts[i].as_str() {
                    "-a" | "--all" => all = true,
                    "-l" | "--long" => long = true,
                    "-la" | "-al" => {
                        all = true;
                        long = true;
                    }
                    "--re" => re = true,
                    "--re-deep" => {
                        re = true;
                        recursive = true;
                    }
                    "--re-insensitive" => re_insensitive = true,
                    "--xcaps" => re_insensitive = true,
                    "-tag" | "--tags" => show_tags = true,
                    "-t" | "--tag" => {
                        if i + 1 < parts.len() {
                            if parts[i+1] == "--deep" && i + 2 < parts.len() {
                                recursive = true;
                                tag_pattern_strs.push(parts[i+2].clone());
                                i += 2;
                            } else {
                                tag_pattern_strs.push(parts[i+1].clone());
                                i += 1;
                            }
                        } else {
                            return Err("标签查询参数需要指定匹配模式，用法：ls -t <标签正则>".into());
                        }
                    }
                    p => path = Some(alias_manager.resolve_path(p)),
                }
                i += 1;
            }
            
            let mut tag_patterns = Vec::new();
            for pattern_str in tag_pattern_strs {
                match regex::Regex::new(&pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("标签正则表达式无效: {}", e).into()),
                }
            }
            
            let (display, raw) = ls::cmd_ls(all, long, re, re_insensitive, show_tags, recursive, path.as_deref(), tag_manager, &tag_patterns)?;
            Ok((false, display, raw))
        }
        "open" => {
            let path = if let Some(p) = parts.get(1) {
                p.clone()
            } else if !input_data.is_empty() {
                input_data.to_string()
            } else {
                return Err("Usage: open <file>".into());
            };
            let resolved_path = alias_manager.resolve_path(&path);
            let (display, raw) = open::cmd_open(&resolved_path)?;
            Ok((false, display, raw))
        }
        "mv" => {
            let mut source: Option<String> = None;
            let mut destination: Option<String> = None;
            let mut copy = false;

            for part in &parts[1..] {
                if part == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(alias_manager.resolve_path(part));
                } else if destination.is_none() {
                    destination = Some(alias_manager.resolve_path(part));
                }
            }

            let source = source.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;
            let destination = destination.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;
            
            let (display, raw) = mv::cmd_mv(&source, &destination, copy)?;
            Ok((false, display, raw))
        }
        "alias" => {
            let alias_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = alias::cmd_alias(alias_manager, &alias_args)?;
            Ok((false, display, raw))
        }
        "tag" | "t" => {
            let tag_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = tag::cmd_tag(tag_manager, &tag_args)?;
            Ok((false, display, raw))
        }
        "exit" | "quit" | "q" => {
            Ok((true, "👋 Goodbye!".bright_green().to_string(), String::new()))
        }
        "clear" | "cls" => {
            let (display, raw) = clear::cmd_clear()?;
            Ok((false, display, raw))
        }
        "help" | "?" => {
            let (display, raw) = help::cmd_help()?;
            Ok((false, display, raw))
        }
        "mkdf" => {
            let mkdf_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = mkdf::cmd_mkdf(&mkdf_args)?;
            Ok((false, display, raw))
        }
        _ => {
            let display = format!(
                "{} Command not found: {}. Type 'help' for available commands.",
                "❌".red(),
                cmd.cyan()
            );
            Ok((false, display, String::new()))
        }
    }
}

fn execute_command(input: &str, alias_manager: &mut AliasManager, tag_manager: &mut TagManager) -> Result<bool, Box<dyn std::error::Error>> {
    let input = input.replace("\n", " ");
    let command_segments: Vec<&str> = input.split("->").map(|s| s.trim()).collect();
    
    let mut previous_raw_data = String::new();
    let mut should_exit = false;

    for segment in command_segments.iter() {
        if segment.is_empty() {
            continue;
        }

        let continue_on_error = segment.ends_with('!');
        let cmd = if continue_on_error { &segment[..segment.len()-1] } else { segment };
        
        let cmd = if cmd.contains("{}") {
            cmd.replace("{}", &previous_raw_data)
        } else {
            cmd.to_string()
        };

        match execute_single_command(&cmd, &previous_raw_data, alias_manager, tag_manager) {
            Ok((exit, display_output, raw_data)) => {
                println!("{}", display_output);
                previous_raw_data = raw_data;
                if exit {
                    should_exit = true;
                    break;
                }
            }
            Err(e) => {
                let error_msg = format!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                println!("{}", error_msg);
                previous_raw_data = String::new();
                if !continue_on_error {
                    return Err(error_msg.into());
                }
            }
        }
    }

    Ok(should_exit)
}

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    print_welcome();

    let mut alias_manager = AliasManager::new()?;
    let mut tag_manager = TagManager::new()?;
    
    let helper = RfeHelper {
        completer: FilenameCompleter::new(),
        alias_manager: AliasManager::new()?,
        tag_manager: TagManager::new()?,
    };

    let mut rl = rustyline::Editor::new()?;
    
    rl.bind_sequence(
        KeyEvent(KeyCode::Esc, rustyline::Modifiers::NONE),
        Cmd::Kill(Movement::WholeLine),
    );
    
    rl.set_helper(Some(helper));

    loop {
        let prompt = get_prompt_string();
        match rl.readline(&prompt) {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);

                match execute_command(input, &mut alias_manager, &mut tag_manager) {
                    Ok(should_exit) => {
                        if should_exit {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("{}", "👋 Goodbye!".bright_green());
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("{}", "👋 Goodbye!".bright_green());
                break;
            }
            Err(e) => {
                eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        if let Err(e) = run_repl() {
            eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
            std::process::exit(1);
        }
        return Ok(());
    }

    let mut alias_manager = AliasManager::new()?;
    let mut tag_manager = TagManager::new()?;
    let cmd = &args[1].to_lowercase();
    let result = match cmd.as_str() {
        "pwd" => pwd::cmd_pwd(),
        "cppwd" => clipboard::cmd_cppwd(),
        "cpf" => {
            let path = args.get(2).map(|s| s.as_str()).ok_or("Usage: rfe cpf <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            clipboard::cmd_cpf(&resolved_path)
        }
        "cd" => {
            let path = args.get(2).map(|s| s.as_str());
            let resolved_path = path.map(|p| alias_manager.resolve_path(p));
            cd::cmd_cd(resolved_path.as_deref())
        }
        "ls" => {
            let mut all = false;
            let mut long = false;
            let mut re = false;
            let mut re_insensitive = false;
            let mut show_tags = false;
            let mut recursive = false;
            let mut path: Option<String> = None;
            let mut tag_pattern_strs: Vec<String> = Vec::new();
            
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "-a" | "--all" => all = true,
                    "-l" | "--long" => long = true,
                    "-la" | "-al" => {
                        all = true;
                        long = true;
                    }
                    "--re" => re = true,
                    "--re-deep" => {
                        re = true;
                        recursive = true;
                    }
                    "--re-insensitive" => re_insensitive = true,
                    "--xcaps" => re_insensitive = true,
                    "-tag" | "--tags" => show_tags = true,
                    "-t" | "--tag" => {
                        if i + 1 < args.len() {
                            if args[i+1] == "--deep" && i + 2 < args.len() {
                                recursive = true;
                                tag_pattern_strs.push(args[i+2].clone());
                                i += 2;
                            } else {
                                tag_pattern_strs.push(args[i+1].clone());
                                i += 1;
                            }
                        } else {
                            return Err("标签查询参数需要指定匹配模式，用法：ls -t <标签正则>".into());
                        }
                    }
                    p => path = Some(alias_manager.resolve_path(p)),
                }
                i += 1;
            }
            
            let mut tag_patterns = Vec::new();
            for pattern_str in tag_pattern_strs {
                match regex::Regex::new(&pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("标签正则表达式无效: {}", e).into()),
                }
            }
            
            ls::cmd_ls(all, long, re, re_insensitive, show_tags, recursive, path.as_deref(), &tag_manager, &tag_patterns)
        }
        "open" => {
            let path = args.get(2).map(|s| s.as_str()).ok_or("Usage: rfe open <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            open::cmd_open(&resolved_path)
        }
        "mv" => {
            let mut source: Option<String> = None;
            let mut destination: Option<String> = None;
            let mut copy = false;

            for arg in &args[2..] {
                if arg == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(alias_manager.resolve_path(arg));
                } else if destination.is_none() {
                    destination = Some(alias_manager.resolve_path(arg));
                }
            }

            let source = source.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            let destination = destination.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            
            mv::cmd_mv(&source, &destination, copy)
        }
        "alias" => {
            let alias_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            alias::cmd_alias(&mut alias_manager, &alias_args)
        }
        "tag" | "t" => {
            let tag_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            tag::cmd_tag(&mut tag_manager, &tag_args)
        }
        "exit" => {
            Ok(("👋 Goodbye!".bright_green().to_string(), String::new()))
        }
        "clear" => clear::cmd_clear(),
        "help" => help::cmd_help(),
        "mkdf" => {
            let mkdf_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            mkdf::cmd_mkdf(&mkdf_args)
        }
        _ => {
            Ok((format!(
                "{} Command not found: {}. Type 'rfe help' for available commands.",
                "❌".red(),
                cmd.cyan()
            ), String::new()))
        }
    };

    match result {
        Ok((output, _raw)) => println!("{}", output),
        Err(e) => {
            eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
            std::process::exit(1);
        }
    }

    Ok(())
}