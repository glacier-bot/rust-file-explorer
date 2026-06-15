use std::env;

mod app;
mod cache;
mod commands;
mod completion;
mod helper;
mod managers;
mod messaging;
mod models;
mod resolver;
mod utils;

use crate::managers::{alias::AliasManager, tag::TagManager};
use crate::utils::moe;

/// 运行直接命令模式（非 REPL）
fn run_direct_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
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
            let (display, raw) = commands::pwd::cmd_pwd()?;
            Ok((display, raw))
        }
        "cppwd" => {
            let (display, raw) = commands::clipboard::cmd_cppwd()?;
            Ok((display, raw))
        }
        "cpf" => {
            let path = args
                .get(arg_offset + 1)
                .map(|s| s.as_str())
                .ok_or("Usage: rfe cpf <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            commands::clipboard::cmd_cpf(&resolved_path)
        }
        "cd" => {
            let mut is_idx = false;
            let mut idx_tag: Option<String> = None;
            let mut path: Option<String> = None;

            let mut i = arg_offset + 1;
            while i < args.len() {
                match args[i].as_str() {
                    "-idx" => {
                        is_idx = true;
                        if i + 1 < args.len() {
                            idx_tag = Some(args[i + 1].clone());
                            i += 1;
                        }
                    }
                    p => path = Some(alias_manager.resolve_path(p)),
                }
                i += 1;
            }

            if is_idx {
                match commands::cd::cmd_cd(
                    None,
                    None,
                    true,
                    idx_tag.as_deref(),
                    Some(&tag_manager),
                    None,
                )? {
                    commands::cd::CdResult::Success(display, raw, _) => Ok((display, raw)),
                    commands::cd::CdResult::NeedSelection(items) => {
                        let output = commands::cd::render_selection_list(&items);
                        println!("{}", output);
                        messaging::print_cd_selection_prompt();
                        let _ = std::io::Write::flush(&mut std::io::stdout());

                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        let selection: usize = match input.trim().parse() {
                            Ok(n) => n,
                            Err(_) => {
                                return Err("Invalid input, please enter a number.".into());
                            }
                        };

                        if selection < 1 || selection > items.len() {
                            return Err(format!(
                                "Selection out of range, please enter a number between 1 and {}.",
                                items.len()
                            )
                            .into());
                        }

                        let item = &items[selection - 1];
                        let target = std::path::PathBuf::from(&item.full_path);

                        if !target.exists() {
                            return Err(format!(
                                "Directory does not exist or is not accessible: {}",
                                target.display()
                            )
                            .into());
                        }

                        env::set_current_dir(&target)?;
                        let plain_path = target.display().to_string();
                        let display = messaging::format_changed_to(&plain_path);
                        Ok((display, plain_path))
                    }
                }
            } else {
                match commands::cd::cmd_cd(path.as_deref(), None, false, None, None, None)? {
                    commands::cd::CdResult::Success(display, raw, _) => Ok((display, raw)),
                    commands::cd::CdResult::NeedSelection(_) => Err("Unexpected error".into()),
                }
            }
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
                                "Tag query parameter requires a pattern, usage: ls -t <tag_regex>"
                                    .into(),
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
                    Err(e) => return Err(format!("Invalid tag regex: {}", e).into()),
                }
            }

            commands::ls::cmd_ls(
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
            .map(|(display, raw, _items)| (display, raw))
        }
        "open" => {
            let path = args
                .get(arg_offset + 1)
                .map(|s| s.as_str())
                .ok_or("Usage: rfe open <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            commands::open::cmd_open(&resolved_path)
        }
        "mv" => {
            let mut source: Option<String> = None;
            let mut destination: Option<String> = None;
            let mut copy = false;

            let mut i = arg_offset + 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--cp" => {
                        copy = true;
                        i += 1;
                    }
                    "-r" => {
                        if i + 1 >= args.len() {
                            return Err("Missing path after -r parameter".into());
                        }
                        return Err("-r parameter is only available in REPL mode (interactive mode)".into());
                    }
                    part => {
                        let resolved = alias_manager.resolve_path(part);
                        if source.is_none() {
                            source = Some(resolved);
                        } else if destination.is_none() {
                            destination = Some(resolved);
                        }
                        i += 1;
                    }
                }
            }

            let source = source.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            let destination =
                destination.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;

            commands::mv::cmd_mv(&source, &destination, copy)
        }
        "alias" => {
            let alias_args: Vec<&str> = args[arg_offset + 1..].iter().map(|s| s.as_str()).collect();
            commands::alias::cmd_alias(&mut alias_manager, &alias_args)
        }
        "tag" | "t" => {
            let tag_args: Vec<&str> = args[arg_offset + 1..].iter().map(|s| s.as_str()).collect();
            commands::tag::cmd_tag(&mut tag_manager, &tag_args)
        }
        "exit" => {
            let display = messaging::format_exit_message();
            Ok((display, String::new()))
        }
        "clear" => commands::clear::cmd_clear(),
        "help" => commands::help::cmd_help(),
        "welcome" => commands::welcome::cmd_welcome(),
        "mkdf" => {
            let mkdf_args: Vec<&str> = args[arg_offset + 1..].iter().map(|s| s.as_str()).collect();
            commands::mkdf::cmd_mkdf(&mkdf_args)
        }
        "change" => {
            let change_args: Vec<&str> =
                args[arg_offset + 1..].iter().map(|s| s.as_str()).collect();
            commands::change::cmd_change(&change_args)
        }
        _ => {
            let display = messaging::format_command_not_found_cli(cmd);
            Ok((display, String::new()))
        }
    };

    match result {
        Ok((output, _raw)) => println!("{}", output),
        Err(e) => {
            messaging::print_error(&e.to_string());
            std::process::exit(1);
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        if let Err(e) = app::repl::run_repl() {
            messaging::print_error(&e.to_string());
            std::process::exit(1);
        }
        return Ok(());
    }

    if args[1] == "-moe" || args[1] == "--moe" {
        moe::enable_moe();
        if args.len() == 2 {
            if let Err(e) = app::repl::run_repl() {
                messaging::print_error(&e.to_string());
                std::process::exit(1);
            }
            return Ok(());
        }
    }

    run_direct_command(args)
}
