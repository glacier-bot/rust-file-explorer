//! CLI 命令解析与执行模块
//! 负责解析单条命令并调用对应的命令处理函数

use crate::app::exec::{exec_cd, exec_cpf, exec_ls, exec_mv, exec_open, ExecContext};
use crate::app::pipeline::CommandResult;

use crate::managers::{alias::AliasManager, tag::TagManager};
use crate::models::FileInfo;
use crate::utils::split_command_args;
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

    let ctx = ExecContext {
        input,
        input_data,
        parts: &parts,
        alias_manager,
        tag_manager,
        last_ls_items,
        previous_dir,
    };

    match cmd.as_str() {
        "pwd" => {
            let (display, raw) = crate::commands::pwd::cmd_pwd()?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "cppwd" => {
            let (display, raw) = crate::commands::clipboard::cmd_cppwd()?;
            Ok((CommandResult::Normal(false), display, raw, None))
        }
        "cpf" => exec_cpf(&ctx),
        "cd" => exec_cd(&ctx),
        "ls" => exec_ls(&ctx),
        "open" => exec_open(&ctx),
        "mv" => exec_mv(&ctx),
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
