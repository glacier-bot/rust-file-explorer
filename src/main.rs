use colored::Colorize;
use rustyline::{Cmd, KeyCode, KeyEvent, Movement};
use std::env;
use std::sync::{Arc, Mutex};

mod cache;
mod commands;
mod helper;
mod managers;
mod models;
mod utils;

use crate::utils::moe::{self, is_moe};

use crate::utils::split_command_args;

use crate::commands::*;
use crate::helper::RfeHelper;
use crate::managers::{alias::AliasManager, tag::TagManager};
use rustyline::completion::FilenameCompleter;

fn print_welcome() {
    let (display, _) = welcome::cmd_welcome().unwrap_or_default();
    println!("{}", display);
}

fn get_prompt_string() -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    let dir_str = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");

    if is_moe() {
        format!("rfe 🌸 {} 💖 >", dir_str)
    } else {
        format!("rfe {} >", dir_str)
    }
}

fn execute_single_command(
    input: &str,
    input_data: &str,
    alias_manager: &Arc<Mutex<AliasManager>>,
    tag_manager: &Arc<Mutex<TagManager>>,
    previous_dir: Option<&str>,
) -> Result<(bool, String, String, Option<String>), Box<dyn std::error::Error>> {
    let parts: Vec<String> = split_command_args(input);

    if parts.is_empty() {
        return Ok((false, String::new(), String::new(), None));
    }

    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "pwd" => {
            let (display, raw) = pwd::cmd_pwd()?;
            Ok((false, display, raw, None))
        }
        "cppwd" => {
            let (display, raw) = clipboard::cmd_cppwd()?;
            Ok((false, display, raw, None))
        }
        "cpf" => {
            let path = if let Some(p) = parts.get(1) {
                p.clone()
            } else if !input_data.is_empty() {
                input_data.to_string()
            } else {
                return Err("Usage: cpf <file>".into());
            };
            let resolved_path = alias_manager.lock().unwrap().resolve_path(&path);
            let (display, raw) = clipboard::cmd_cpf(&resolved_path)?;
            Ok((false, display, raw, None))
        }
        "cd" => {
            let path = if parts.len() > 1 {
                Some(alias_manager.lock().unwrap().resolve_path(&parts[1]))
            } else if !input_data.is_empty() {
                Some(input_data.to_string())
            } else {
                None
            };
            let (display, raw, new_prev) = cd::cmd_cd(path.as_deref(), previous_dir)?;
            Ok((false, display, raw, new_prev))
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
                            if parts[i + 1] == "--deep" && i + 2 < parts.len() {
                                recursive = true;
                                tag_pattern_strs.push(parts[i + 2].clone());
                                i += 2;
                            } else {
                                tag_pattern_strs.push(parts[i + 1].clone());
                                i += 1;
                            }
                        } else {
                            return Err(
                                "标签查询参数需要指定匹配模式，用法：ls -t <标签正则>".into()
                            );
                        }
                    }
                    p => path = Some(alias_manager.lock().unwrap().resolve_path(p)),
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

            let (display, raw) = ls::cmd_ls(
                all,
                long,
                re,
                re_insensitive,
                show_tags,
                recursive,
                path.as_deref(),
                &tag_manager.lock().unwrap(),
                &tag_patterns,
            )?;
            Ok((false, display, raw, None))
        }
        "open" => {
            let path = if let Some(p) = parts.get(1) {
                p.clone()
            } else if !input_data.is_empty() {
                input_data.to_string()
            } else {
                return Err("Usage: open <file>".into());
            };
            let resolved_path = alias_manager.lock().unwrap().resolve_path(&path);
            let (display, raw) = open::cmd_open(&resolved_path)?;
            Ok((false, display, raw, None))
        }
        "mv" => {
            let mut source: Option<String> = None;
            let mut destination: Option<String> = None;
            let mut copy = false;

            for part in &parts[1..] {
                if part == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(alias_manager.lock().unwrap().resolve_path(part));
                } else if destination.is_none() {
                    destination = Some(alias_manager.lock().unwrap().resolve_path(part));
                }
            }

            let source = source.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;
            let destination =
                destination.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;

            let (display, raw) = mv::cmd_mv(&source, &destination, copy)?;
            Ok((false, display, raw, None))
        }
        "alias" => {
            let alias_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = alias::cmd_alias(&mut alias_manager.lock().unwrap(), &alias_args)?;
            Ok((false, display, raw, None))
        }
        "tag" | "t" => {
            let tag_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = tag::cmd_tag(&mut tag_manager.lock().unwrap(), &tag_args)?;
            Ok((false, display, raw, None))
        }
        "exit" | "quit" | "q" => {
            if is_moe() {
                Ok((
                    true,
                    "👋🌸💖 Bye-bye! See you next time~ 💕"
                        .truecolor(255, 182, 193)
                        .to_string(),
                    String::new(),
                    None,
                ))
            } else {
                Ok((
                    true,
                    "👋 Goodbye!".bright_green().to_string(),
                    String::new(),
                    None,
                ))
            }
        }
        "clear" | "cls" => {
            let (display, raw) = clear::cmd_clear()?;
            Ok((false, display, raw, None))
        }
        "help" | "?" => {
            let (display, raw) = help::cmd_help()?;
            Ok((false, display, raw, None))
        }
        "welcome" => {
            let (display, raw) = welcome::cmd_welcome()?;
            Ok((false, display, raw, None))
        }
        "mkdf" => {
            let mkdf_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = mkdf::cmd_mkdf(&mkdf_args)?;
            Ok((false, display, raw, None))
        }
        _ => {
            if is_moe() {
                let display = format!(
                    "{} {} Command not found: {}. Type 'help' for available commands~ 💕",
                    "😢".truecolor(255, 105, 180),
                    "💔".truecolor(255, 105, 180),
                    cmd.truecolor(255, 182, 193)
                );
                Ok((false, display, String::new(), None))
            } else {
                let display = format!(
                    "{} Command not found: {}. Type 'help' for available commands.",
                    "❌".red(),
                    cmd.cyan()
                );
                Ok((false, display, String::new(), None))
            }
        }
    }
}

fn execute_command(
    input: &str,
    alias_manager: &Arc<Mutex<AliasManager>>,
    tag_manager: &Arc<Mutex<TagManager>>,
    current_previous_dir: &mut Option<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let input = input.replace("\n", " ");
    let command_segments: Vec<&str> = input.split("->").map(|s| s.trim()).collect();

    let mut previous_raw_data = String::new();
    let mut should_exit = false;

    for segment in command_segments.iter() {
        if segment.is_empty() {
            continue;
        }

        let continue_on_error = segment.ends_with('!');
        let cmd = if continue_on_error {
            &segment[..segment.len() - 1]
        } else {
            segment
        };

        let cmd = if cmd.contains("{}") {
            cmd.replace("{}", &previous_raw_data)
        } else {
            cmd.to_string()
        };

        match execute_single_command(
            &cmd,
            &previous_raw_data,
            alias_manager,
            tag_manager,
            current_previous_dir.as_deref(),
        ) {
            Ok((exit, display_output, raw_data, new_prev_dir)) => {
                println!("{}", display_output);
                previous_raw_data = raw_data;
                if let Some(new_prev) = new_prev_dir {
                    *current_previous_dir = Some(new_prev);
                }
                if exit {
                    should_exit = true;
                    break;
                }
            }
            Err(e) => {
                let error_msg = if is_moe() {
                    format!(
                        "{} {} {}",
                        "😢💔".truecolor(255, 105, 180),
                        "Error:".truecolor(255, 105, 180),
                        e.to_string().truecolor(255, 182, 193)
                    )
                } else {
                    format!("{} {}", "❌ Error:".red(), e.to_string().bright_red())
                };
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

    let alias_manager = Arc::new(Mutex::new(AliasManager::new()?));
    let tag_manager = Arc::new(Mutex::new(TagManager::new()?));
    let mut previous_dir: Option<String> = None;

    let helper = RfeHelper {
        completer: FilenameCompleter::new(),
        alias_manager: Arc::clone(&alias_manager),
        tag_manager: Arc::clone(&tag_manager),
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

                match execute_command(input, &alias_manager, &tag_manager, &mut previous_dir) {
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

    if args[1] == "-moe" || args[1] == "--moe" {
        moe::enable_moe();
        if args.len() == 2 {
            if let Err(e) = run_repl() {
                eprintln!(
                    "{} {} {}",
                    "😢💔".truecolor(255, 105, 180),
                    "Error:".truecolor(255, 105, 180),
                    e.to_string().truecolor(255, 182, 193)
                );
                std::process::exit(1);
            }
            return Ok(());
        }
    }

    let mut alias_manager = AliasManager::new()?;
    let mut tag_manager = TagManager::new()?;

    let (cmd, arg_offset) = if args[1] == "-moe" || args[1] == "--moe" {
        moe::enable_moe();
        (&args[2].to_lowercase(), 2)
    } else {
        (&args[1].to_lowercase(), 1)
    };
    let result = match cmd.as_str() {
        "pwd" => {
            let (display, raw) = pwd::cmd_pwd()?;
            Ok((display, raw))
        }
        "cppwd" => {
            let (display, raw) = clipboard::cmd_cppwd()?;
            Ok((display, raw))
        }
        "cpf" => {
            let path = args
                .get(arg_offset + 1)
                .map(|s| s.as_str())
                .ok_or("Usage: rfe cpf <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            clipboard::cmd_cpf(&resolved_path)
        }
        "cd" => {
            let path = args.get(arg_offset + 1).map(|s| s.as_str());
            let resolved_path = path.map(|p| alias_manager.resolve_path(p));
            let (display, raw, _) = cd::cmd_cd(resolved_path.as_deref(), None)?;
            Ok((display, raw))
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

            let mut i = arg_offset + 1;
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
                            if args[i + 1] == "--deep" && i + 2 < args.len() {
                                recursive = true;
                                tag_pattern_strs.push(args[i + 2].clone());
                                i += 2;
                            } else {
                                tag_pattern_strs.push(args[i + 1].clone());
                                i += 1;
                            }
                        } else {
                            return Err(
                                "标签查询参数需要指定匹配模式，用法：ls -t <标签正则>".into()
                            );
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

            ls::cmd_ls(
                all,
                long,
                re,
                re_insensitive,
                show_tags,
                recursive,
                path.as_deref(),
                &tag_manager,
                &tag_patterns,
            )
        }
        "open" => {
            let path = args
                .get(arg_offset + 1)
                .map(|s| s.as_str())
                .ok_or("Usage: rfe open <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            open::cmd_open(&resolved_path)
        }
        "mv" => {
            let mut source: Option<String> = None;
            let mut destination: Option<String> = None;
            let mut copy = false;

            for arg in &args[arg_offset + 1..] {
                if arg == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(alias_manager.resolve_path(arg));
                } else if destination.is_none() {
                    destination = Some(alias_manager.resolve_path(arg));
                }
            }

            let source = source.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            let destination =
                destination.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;

            mv::cmd_mv(&source, &destination, copy)
        }
        "alias" => {
            let alias_args: Vec<&str> = args[arg_offset + 1..].iter().map(|s| s.as_str()).collect();
            alias::cmd_alias(&mut alias_manager, &alias_args)
        }
        "tag" | "t" => {
            let tag_args: Vec<&str> = args[arg_offset + 1..].iter().map(|s| s.as_str()).collect();
            tag::cmd_tag(&mut tag_manager, &tag_args)
        }
        "exit" => {
            if is_moe() {
                Ok((
                    "👋🌸💖 Bye-bye! See you next time~ 💕"
                        .truecolor(255, 182, 193)
                        .to_string(),
                    String::new(),
                ))
            } else {
                Ok(("👋 Goodbye!".bright_green().to_string(), String::new()))
            }
        }
        "clear" => clear::cmd_clear(),
        "help" => help::cmd_help(),
        "welcome" => welcome::cmd_welcome(),
        "mkdf" => {
            let mkdf_args: Vec<&str> = args[arg_offset + 1..].iter().map(|s| s.as_str()).collect();
            mkdf::cmd_mkdf(&mkdf_args)
        }
        _ => {
            if is_moe() {
                Ok((
                    format!(
                        "{} {} Command not found: {}. Type 'rfe help' for available commands~ 💕",
                        "😢".truecolor(255, 105, 180),
                        "💔".truecolor(255, 105, 180),
                        cmd.truecolor(255, 182, 193)
                    ),
                    String::new(),
                ))
            } else {
                Ok((
                    format!(
                        "{} Command not found: {}. Type 'rfe help' for available commands.",
                        "❌".red(),
                        cmd.cyan()
                    ),
                    String::new(),
                ))
            }
        }
    };

    match result {
        Ok((output, _raw)) => println!("{}", output),
        Err(e) => {
            if is_moe() {
                eprintln!(
                    "{} {} {}",
                    "😢💔".truecolor(255, 105, 180),
                    "Error:".truecolor(255, 105, 180),
                    e.to_string().truecolor(255, 182, 193)
                );
            } else {
                eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

// todo: mv命令批量操作，移动和复制过程可视化
