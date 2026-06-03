//! 命令管道执行模块
//! 负责解析并执行命令管道（如 `ls -> cd {}`），处理占位符展开和错误传播

use crate::app::cli::execute_single_command;
use crate::messaging;
use crate::models::FileInfo;
use crate::utils::path::expand_pop_placeholders;
use std::sync::{Arc, Mutex};

/// 命令执行结果
#[derive(Debug)]
pub enum CommandResult {
    /// 正常执行，bool 表示是否应退出
    Normal(bool),
    /// 需要 cd 选择
    NeedCdSelection(Vec<crate::commands::cd::CdSelectionItem>),
}

/// 执行命令管道（支持 `->` 连接多个命令）
///
/// # Arguments
/// * `input` - 用户输入的命令字符串
/// * `alias_manager` - 别名管理器
/// * `tag_manager` - 标签管理器
/// * `last_ls_items` - 上一次 ls 的结果列表
/// * `current_previous_dir` - 当前的上一个目录
///
/// # Returns
/// * 最终的命令执行结果
pub fn execute_command(
    input: &str,
    alias_manager: &Arc<Mutex<crate::managers::alias::AliasManager>>,
    tag_manager: &Arc<Mutex<crate::managers::tag::TagManager>>,
    last_ls_items: &Arc<Mutex<Vec<FileInfo>>>,
    current_previous_dir: &mut Option<String>,
) -> Result<CommandResult, Box<dyn std::error::Error>> {
    let input = input.replace("\n", " ");
    let command_segments: Vec<&str> = input.split("->").map(|s| s.trim()).collect();

    let mut previous_raw_data = String::new();
    let mut result = CommandResult::Normal(false);

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

        let mut cmd = cmd.to_string();
        if cmd.contains("{}") {
            let expand = expand_pop_placeholders(&cmd, &previous_raw_data);
            if expand.reached_boundary {
                messaging::print_path_boundary_warning(expand.actual_pops, &previous_raw_data);
            }
            cmd = expand.expanded;
        }

        match execute_single_command(
            &cmd,
            &previous_raw_data,
            alias_manager,
            tag_manager,
            last_ls_items,
            current_previous_dir.as_deref(),
        ) {
            Ok((cmd_result, display_output, raw_output, new_prev_dir)) => {
                println!("{}", display_output);
                if let CommandResult::NeedCdSelection(_) = cmd_result {
                    return Ok(cmd_result);
                }
                if let CommandResult::Normal(exit) = cmd_result {
                    result = CommandResult::Normal(exit);
                }
                previous_raw_data = raw_output;
                if let Some(new_prev) = new_prev_dir {
                    *current_previous_dir = Some(new_prev);
                }
            }
            Err(e) => {
                let error_msg = messaging::format_error(&e.to_string());
                println!("{}", error_msg);
                previous_raw_data = String::new();
                if !continue_on_error {
                    return Err(error_msg.into());
                }
            }
        }
    }

    Ok(result)
}
