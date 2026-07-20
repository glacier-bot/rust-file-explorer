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
    /// 需要 open 选择
    NeedOpenSelection(Vec<crate::commands::cd::CdSelectionItem>),
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
    
    let mut segments = Vec::new();
    let mut last = 0;
    let chars: Vec<char> = input.chars().collect();
    
    while last < chars.len() {
        let mut found = false;
        for i in last..chars.len() {
            if i + 1 < chars.len() && chars[i] == '-' && chars[i + 1] == '>' {
                let continue_on_error = i + 2 < chars.len() && chars[i + 2] == '!';
                let segment_end = if continue_on_error { i + 3 } else { i + 2 };
                let segment: String = chars[last..i].iter().collect();
                let segment = segment.trim().to_string();
                if !segment.is_empty() {
                    segments.push((segment, continue_on_error));
                }
                last = segment_end;
                found = true;
                break;
            }
        }
        if !found {
            let segment: String = chars[last..].iter().collect();
            let segment = segment.trim().to_string();
            if !segment.is_empty() {
                segments.push((segment, false));
            }
            break;
        }
    }

    let mut previous_raw_data = String::new();
    let mut result = CommandResult::Normal(false);

    for (cmd, continue_on_error) in segments {

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
                if let CommandResult::NeedOpenSelection(_) = cmd_result {
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
