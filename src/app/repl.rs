//! REPL 交互模块
//! 负责交互式命令行循环、输入读取、历史记录和 cd 选择处理

use crate::app::pipeline::{execute_command, CommandResult};
use crate::completion::CompletionManager;
use crate::helper::RfeHelper;
use crate::managers::{alias::AliasManager, tag::TagManager};
use crate::messaging;
use crate::models::FileInfo;
use colored::Colorize;
use rustyline::completion::FilenameCompleter;
use rustyline::{Cmd, KeyCode, KeyEvent, Movement};
use std::env;
use std::sync::{Arc, Mutex};

/// 获取 REPL 提示符字符串
fn get_prompt_string() -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    let dir_str = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");

    if crate::utils::moe::is_moe() {
        format!("rfe 🌸 {} 💖 >", dir_str)
    } else {
        format!("rfe {} >", dir_str)
    }
}

/// 运行 REPL 交互循环
///
/// # Returns
/// * Ok(()) 表示正常退出
/// * Err 表示发生错误
pub fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    messaging::print_welcome();

    let alias_manager = Arc::new(Mutex::new(AliasManager::new()?));
    let tag_manager = Arc::new(Mutex::new(TagManager::new()?));
    let last_ls_items: Arc<Mutex<Vec<FileInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let mut previous_dir: Option<String> = None;
    let mut pending_cd_selection: Option<(Vec<crate::commands::cd::CdSelectionItem>, Option<String>)> = None;
    let mut pending_open_selection: Option<(Vec<crate::commands::cd::CdSelectionItem>, Option<String>)> = None;

    let helper = RfeHelper {
        completer: FilenameCompleter::new(),
        alias_manager: Arc::clone(&alias_manager),
        tag_manager: Arc::clone(&tag_manager),
        last_ls_items: Arc::clone(&last_ls_items),
        completion_manager: CompletionManager::new(),
    };

    let mut rl = rustyline::Editor::new()?;

    rl.bind_sequence(
        KeyEvent(KeyCode::Esc, rustyline::Modifiers::NONE),
        Cmd::Kill(Movement::WholeLine),
    );

    // 右方向键：如果光标在行尾则接受提示，否则正常移动
    // 注意：rustyline 默认右方向键在行尾时接受内联提示
    // 如果需要自定义行为，可以通过 Cmd::MoveForwardChar 或其他方式处理

    rl.set_helper(Some(helper));

    loop {
        let prompt = if pending_cd_selection.is_some() || pending_open_selection.is_some() {
            format!("{} Enter selection number: ", "📍".bright_blue())
        } else {
            get_prompt_string()
        };

        match rl.readline(&prompt) {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);

                if let Some((items, _tag)) = pending_cd_selection.take() {
                    let selection: usize = match input.parse() {
                        Ok(n) => n,
                        Err(_) => {
                            eprintln!("{} Invalid input, please enter a number.", "❌".red());
                            continue;
                        }
                    };

                    if selection < 1 || selection > items.len() {
                        eprintln!(
                            "{} Selection out of range, please enter a number between 1 and {}.",
                            "❌".red(),
                            items.len()
                        );
                        continue;
                    }

                    let item = &items[selection - 1];
                    let target = std::path::PathBuf::from(&item.full_path);

                    if !target.exists() {
                        eprintln!(
                            "{} Directory does not exist or is not accessible: {}",
                            "❌".red(),
                            target.display()
                        );
                        continue;
                    }

                    let current_dir = env::current_dir()?;
                    let new_previous_dir = if target != current_dir {
                        Some(current_dir.display().to_string())
                    } else {
                        None
                    };

                    env::set_current_dir(&target)?;
                    let plain_path = target.display().to_string();
                    let display = messaging::format_changed_to(&plain_path);
                    println!("{}", display);

                    if let Some(new_prev) = new_previous_dir {
                        previous_dir = Some(new_prev);
                    }
                } else if let Some((items, _tag)) = pending_open_selection.take() {
                    let selection: usize = match input.parse() {
                        Ok(n) => n,
                        Err(_) => {
                            eprintln!("{} Invalid input, please enter a number.", "❌".red());
                            continue;
                        }
                    };

                    if selection < 1 || selection > items.len() {
                        eprintln!(
                            "{} Selection out of range, please enter a number between 1 and {}.",
                            "❌".red(),
                            items.len()
                        );
                        continue;
                    }

                    let item = &items[selection - 1];
                    let target = &item.full_path;

                    match crate::commands::open::cmd_open(target) {
                        Ok((display, _)) => println!("{}", display),
                        Err(e) => eprintln!("{} {}", "❌".red(), e),
                    }
                } else {
                    match execute_command(
                        input,
                        &alias_manager,
                        &tag_manager,
                        &last_ls_items,
                        &mut previous_dir,
                    ) {
                        Ok(CommandResult::Normal(should_exit)) => {
                            if should_exit {
                                break;
                            }
                        }
                        Ok(CommandResult::NeedCdSelection(items)) => {
                            let tag = items.get(0).and_then(|item| item.tags.first().cloned());
                            let output = crate::commands::cd::render_selection_list(&items);
                            println!("{}", output);
                            pending_cd_selection = Some((items, tag));
                        }
                        Ok(CommandResult::NeedOpenSelection(items)) => {
                            let tag = items.get(0).and_then(|item| item.tags.first().cloned());
                            let output = crate::commands::cd::render_selection_list(&items);
                            println!("{}", output);
                            pending_open_selection = Some((items, tag));
                        }
                        Err(e) => {
                            messaging::print_error(&e.to_string());
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                if pending_cd_selection.is_some() {
                    pending_cd_selection = None;
                    messaging::print_selection_cancelled();
                } else if pending_open_selection.is_some() {
                    pending_open_selection = None;
                    messaging::print_selection_cancelled();
                } else {
                    messaging::print_exit_message();
                    break;
                }
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                messaging::print_exit_message();
                break;
            }
            Err(e) => {
                messaging::print_error(&e.to_string());
                break;
            }
        }
    }

    Ok(())
}
