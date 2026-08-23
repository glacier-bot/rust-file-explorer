//! CLI 命令解析与执行模块
//! 负责解析单条命令并调用对应的命令处理函数

use crate::app::pipeline::CommandResult;

use crate::managers::{alias::AliasManager, tag::TagManager};
use crate::models::FileInfo;
use crate::resolver::resolve_line_path;
use crate::utils::split_command_args;
use colored::Colorize;
use std::sync::{Arc, Mutex};

/// 执行单条命令
///
/// # Arguments
/// * `input` - 命令字符串
/// * `input_data` - 上一条命令的原始输出（用于管道）
/// * `alias_manager` - 别名管理器
/// * `tag_manager` - 标签管理器
/// * `last_ls_items` - 上一次 ls 的结果列表
/// * `previous_dir` - 上一个目录
///
/// # Returns
/// * (命令结果, 显示输出, 原始输出, 新的上一个目录)
pub fn execute_single_command(
    input: &str,
    input_data: &str,
    alias_manager: &Arc<Mutex<AliasManager>>,
    tag_manager: &Arc<Mutex<TagManager>>,
    last_ls_items: &Arc<Mutex<Vec<FileInfo>>>,
    previous_dir: Option<&str>,
) -> Result<(CommandResult, String, String, Option<String>), Box<dyn std::error::Error>> {
    let parts: Vec<String> = split_command_args(input);

    if parts.is_empty() {
        return Ok((
            CommandResult::Normal(false),
            String::new(),
            String::new(),
            None,
        ));
    }

    let cmd = parts[0].to_lowercase();

    if cmd == "rfe" {
        let error_msg = crate::messaging::format_error(
            "Already running in rfe. Use 'exit' to quit REPL mode."
        );
        return Ok((CommandResult::Normal(false), error_msg, String::new(), None));
    }

    match cmd.as_str() {
        "pwd" => {
            let (display, raw) = crate::commands::pwd::cmd_pwd()?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "cppwd" => {
            let (display, raw) = crate::commands::clipboard::cmd_cppwd()?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "cpf" => {
            let mut path_parts: Vec<String> = Vec::new();
            let mut i = 1;
            while i < parts.len() {
                if parts[i] == "-r" {
                    if i + 1 >= parts.len() {
                        return Err("Usage: cpf -r <line_number>[/path]".into());
                    }
                     let resolved = resolve_line_path(&parts[i + 1], last_ls_items)?;
                     path_parts.push(resolved);
                     i += 2;
                 } else {
                     let resolved = alias_manager.lock().unwrap().resolve_path(&parts[i]);
                     path_parts.push(resolved);
                     i += 1;
                 }
            }

            let path = if path_parts.is_empty() {
                if !input_data.is_empty() {
                    Some(input_data.to_string())
                } else {
                    None
                }
            } else {
                Some(path_parts.join(" "))
            }
            .ok_or("Usage: cpf <file> or cpf -r <line_number>[/path]")?;

            let (display, raw) = crate::commands::clipboard::cmd_cpf(&path)?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "cd" => {
            let has_shell_ops = input.contains('&') 
                || input.contains('|') 
                || input.contains(';')
                || input.contains('>')
                || input.contains('<')
                || input.contains("&&")
                || input.contains("||");
            
            if has_shell_ops {
                match crate::commands::shell::cmd_shell(input) {
                    Ok((display, raw, new_prev_dir)) => {
                        Ok((CommandResult::Normal(false), display, raw, new_prev_dir))
                    }
                    Err(e) => Err(e),
                }
            } else {
                let mut is_idx = false;
                let mut idx_tag: Option<String> = None;
                let mut path_parts: Vec<String> = Vec::new();
                let mut selection: Option<usize> = None;

                let mut i = 1;
                while i < parts.len() {
                    match parts[i].as_str() {
                        "-tag" => {
                            is_idx = true;
                            if i + 1 < parts.len() {
                                idx_tag = Some(parts[i + 1].clone());
                                i += 1;
                            }
                        }
                        "-sel" => {
                            if i + 1 < parts.len() {
                                if let Ok(n) = parts[i + 1].parse::<usize>() {
                                    selection = Some(n);
                                }
                                i += 1;
                            }
                        }
                        "-r" => {
                             if i + 1 < parts.len() {
                                 let target_path = resolve_line_path(&parts[i + 1], last_ls_items)?;
                                 let target_path_buf = std::path::PathBuf::from(&target_path);

                                if !target_path_buf.exists() {
                                    return Err(format!(
                                        "Path does not exist: {}",
                                        target_path
                                    )
                                    .into());
                                }
                                if !target_path_buf.is_dir() {
                                    return Err(format!(
                                        "'{}' is not a directory",
                                        target_path
                                    )
                                    .into());
                                }

                                let current_dir = std::env::current_dir()?;
                                std::env::set_current_dir(&target_path_buf)?;
                                let new_prev_dir = if target_path_buf != current_dir {
                                    Some(current_dir.display().to_string())
                                } else {
                                    None
                                };
                                let display = format!(
                                    "{} {}",
                                    "Changed to:".green(),
                                    target_path.cyan()
                                );
                                return Ok((
                                    CommandResult::Normal(false),
                                    display,
                                    target_path,
                                    new_prev_dir,
                                ));
                            } else {
                                return Err("Usage: cd -r <line_number>[/sub_path]".into());
                            }
                        }
                    p => {
                         let resolved = alias_manager.lock().unwrap().resolve_path(p);
                         path_parts.push(resolved);
                    }
                    }
                    i += 1;
                }

                let path = if path_parts.is_empty() {
                    None
                } else {
                    Some(path_parts.join(" "))
                };

                if is_idx {
                    match crate::commands::cd::cmd_cd(
                        None,
                        previous_dir,
                        true,
                        idx_tag.as_deref(),
                        Some(&tag_manager.lock().unwrap()),
                        selection,
                    )? {
                        crate::commands::cd::CdResult::Success(display, raw, new_prev) => {
                            Ok((CommandResult::Normal(false), display, raw, new_prev))
                        }
                        crate::commands::cd::CdResult::NeedSelection(items) => Ok((
                            CommandResult::NeedCdSelection(items),
                            String::new(),
                            String::new(),
                            None,
                        )),
                    }
                } else {
                 let path = if path.is_some() {
                         path
                     } else if !input_data.is_empty() {
                         Some(input_data.to_string())
                     } else {
                         None
                     };
                    match crate::commands::cd::cmd_cd(
                        path.as_deref(),
                        previous_dir,
                        false,
                        None,
                        None,
                        None,
                    )? {
                        crate::commands::cd::CdResult::Success(display, raw, new_prev) => {
                            Ok((CommandResult::Normal(false), display, raw, new_prev))
                        }
                        crate::commands::cd::CdResult::NeedSelection(_) => {
                            Err("Unexpected error".into())
                        }
                    }
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
            let mut path_parts: Vec<String> = Vec::new();
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
                                "Tag query parameter requires a pattern, usage: ls -t <tag_regex>"
                                    .into(),
                            );
                        }
                    }
                    p => {
                        let resolved = alias_manager.lock().unwrap().resolve_path(p);
                        path_parts.push(resolved);
                    }
                }
                i += 1;
            }

            let path = if path_parts.is_empty() {
                None
            } else {
                Some(path_parts.join(" "))
            };

            let mut tag_patterns = Vec::new();
            for pattern_str in tag_pattern_strs {
                match regex::Regex::new(&pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("Invalid tag regex: {}", e).into()),
                }
            }

            let (display, raw, items) = crate::commands::ls::cmd_ls(
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
            *last_ls_items.lock().unwrap() = items;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "open" => {
            let mut is_tag = false;
            let mut tag_value: Option<String> = None;
            let mut path_parts: Vec<String> = Vec::new();
            let mut selection: Option<usize> = None;
            let mut i = 1;
            while i < parts.len() {
                match parts[i].as_str() {
                    "-tag" => {
                        is_tag = true;
                        if i + 1 < parts.len() {
                            tag_value = Some(parts[i + 1].clone());
                            i += 1;
                        }
                    }
                    "-sel" => {
                        if i + 1 < parts.len() {
                            if let Ok(n) = parts[i + 1].parse::<usize>() {
                                selection = Some(n);
                            }
                            i += 1;
                        }
                    }
                     "-r" => {
                         if i + 1 >= parts.len() {
                             return Err("Usage: open -r <line_number>[/path]".into());
                         }
                         let resolved = resolve_line_path(&parts[i + 1], last_ls_items)?;
                         path_parts.push(resolved);
                         i += 2;
                     }
                     p => {
                         let resolved = alias_manager.lock().unwrap().resolve_path(p);
                         path_parts.push(resolved);
                    }
                }
                i += 1;
            }

            if is_tag {
                match crate::commands::open::cmd_open_tag(
                    tag_value.as_deref(),
                    Some(&tag_manager.lock().unwrap()),
                    selection,
                )? {
                    crate::commands::open::OpenResult::Success(display, raw) => {
                        Ok((CommandResult::Normal(false), display, raw, None))
                    }
                    crate::commands::open::OpenResult::NeedSelection(items) => Ok((
                        CommandResult::NeedOpenSelection(items),
                        String::new(),
                        String::new(),
                        None,
                    )),
                }
             } else {
                 let path = if path_parts.is_empty() {
                     if !input_data.is_empty() {
                         Some(input_data.to_string())
                     } else {
                         None
                     }
                 } else {
                     Some(path_parts.join(" "))
                 }
                .ok_or("Usage: open <file> or open -r <line_number>[/path] or open -tag <tag>")?;

                let (display, raw) = crate::commands::open::cmd_open(&path)?;
                Ok((CommandResult::Normal(false), display, raw, None))
            }
        }
        "mv" => {
            let mut path_parts: Vec<String> = Vec::new();
            let mut copy = false;

            let mut i = 1;
            while i < parts.len() {
                match parts[i].as_str() {
                    "--cp" => {
                        copy = true;
                        i += 1;
                    }
                     "-r" => {
                         if i + 1 >= parts.len() {
                             return Err("Missing path after -r parameter".into());
                         }
                         let resolved = resolve_line_path(&parts[i + 1], last_ls_items)?;
                         path_parts.push(resolved);
                         i += 2;
                     }
                     part => {
                         let resolved = alias_manager.lock().unwrap().resolve_path(part);
                         path_parts.push(resolved);
                         i += 1;
                     }
                }
            }

            if path_parts.len() < 2 {
                return Err(
                    "Usage: mv <source_path> <destination_path> [--cp] or mv -r <source_line> <destination> or mv <source> -r <destination_line>"
                        .into(),
                );
            }

            let destination = path_parts.pop().unwrap();
            let source = path_parts.join(" ");

            let (display, raw) = crate::commands::mv::cmd_mv(&source, &destination, copy)?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "alias" => {
            let alias_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) =
                crate::commands::alias::cmd_alias(&mut alias_manager.lock().unwrap(), &alias_args)?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "tag" | "t" => {
            let tag_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) =
                crate::commands::tag::cmd_tag(&mut tag_manager.lock().unwrap(), &tag_args)?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "exit" | "quit" | "q" => {
            let display = crate::messaging::format_exit_message();
            Ok((CommandResult::Normal(true), display, String::new(), None))
        }
        "clear" | "cls" => {
            let (display, raw) = crate::commands::clear::cmd_clear()?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "help" | "?" => {
            let (display, raw) = crate::commands::help::cmd_help()?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "welcome" => {
            let (display, raw) = crate::commands::welcome::cmd_welcome()?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "mkdf" => {
            let mkdf_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = crate::commands::mkdf::cmd_mkdf(&mkdf_args)?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "change" => {
            let change_args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();
            let (display, raw) = crate::commands::change::cmd_change(&change_args)?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        _ => {
            match crate::commands::shell::cmd_shell(input) {
                Ok((display, raw, new_prev_dir)) => {
                    Ok((CommandResult::Normal(false), display, raw, new_prev_dir))
                }
                Err(e) => {
                    let display = crate::messaging::format_error(&e.to_string());
                    Ok((CommandResult::Normal(false), display, String::new(), None))
                }
            }
        }
    }
}
