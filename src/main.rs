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

use crate::commands::cli::{
    parse_cd_args, parse_cpf_arg, parse_ls_args, parse_mv_args, parse_open_arg, get_alias_args,
    get_change_args, get_mkdf_args, get_tag_args,
};
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
            let path = parse_cpf_arg(&args, arg_offset, &alias_manager)?;
            commands::clipboard::cmd_cpf(&path)
        }
        "cd" => {
            let cd_args = parse_cd_args(&args, arg_offset, &alias_manager)?;

            if cd_args.is_idx {
                match commands::cd::cmd_cd(
                    None,
                    None,
                    true,
                    cd_args.idx_tag.as_deref(),
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
                match commands::cd::cmd_cd(cd_args.path.as_deref(), None, false, None, None, None)? {
                    commands::cd::CdResult::Success(display, raw, _) => Ok((display, raw)),
                    commands::cd::CdResult::NeedSelection(_) => Err("Unexpected error".into()),
                }
            }
        }
        "ls" => {
            let ls_args = parse_ls_args(&args, arg_offset, &alias_manager)?;
            commands::ls::cmd_ls(
                ls_args.all,
                ls_args.long,
                ls_args.re,
                ls_args.re_insensitive,
                ls_args.show_tags,
                ls_args.recursive,
                ls_args.path.as_deref(),
                &tag_manager,
                &ls_args.tag_patterns,
            )
            .map(|(display, raw, _items)| (display, raw))
        }
        "open" => {
            let path = parse_open_arg(&args, arg_offset, &alias_manager)?;
            commands::open::cmd_open(&path)
        }
        "mv" => {
            let mv_args = parse_mv_args(&args, arg_offset, &alias_manager)?;
            commands::mv::cmd_mv(&mv_args.source, &mv_args.destination, mv_args.copy)
        }
        "alias" => {
            let alias_args = get_alias_args(&args, arg_offset);
            commands::alias::cmd_alias(&mut alias_manager, &alias_args)
        }
        "tag" | "t" => {
            let tag_args = get_tag_args(&args, arg_offset);
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
            let mkdf_args = get_mkdf_args(&args, arg_offset);
            commands::mkdf::cmd_mkdf(&mkdf_args)
        }
        "change" => {
            let change_args = get_change_args(&args, arg_offset);
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
