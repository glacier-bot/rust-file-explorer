//! 内联提示模块
//! 提供命令输入时的内联提示功能

use crate::completion::CompletionManager;

/// 提供输入提示（内联显示，可通过右方向键或 Tab 接受）
/// 返回纯文本字符串，不带 ANSI 颜色代码，避免光标位置计算错误
pub fn hint(
    line: &str,
    pos: usize,
    completion_manager: &CompletionManager,
) -> Option<String> {
    // 空行不显示提示
    if line.is_empty() || pos == 0 {
        return None;
    }

    let context = completion_manager.parse_input_for_completion(line, pos);

    match context {
        // 命令名提示：显示第一个匹配的命令作为内联提示
        crate::completion::CompletionContext::CommandName(ref prefix) if !prefix.is_empty() => {
            let completions = completion_manager.get_command_completions(prefix);
            if let Some((name, _desc)) = completions.first() {
                let hint = if name.starts_with(prefix) {
                    name[prefix.len()..].to_string()
                } else {
                    name.clone()
                };
                Some(hint)
            } else {
                None
            }
        }

        // 命令参数提示：显示第一个匹配的参数作为内联提示
        crate::completion::CompletionContext::CommandArg(ref cmd_name, ref arg_prefix) => {
            if let Some(cmd) = completion_manager.get_command(cmd_name) {
                let completions = cmd.get_arg_completions(arg_prefix);
                if let Some((name, _desc)) = completions.first() {
                    let hint = if arg_prefix.is_empty() {
                        name.clone()
                    } else if name.starts_with(arg_prefix) {
                        name[arg_prefix.len()..].to_string()
                    } else {
                        name.clone()
                    };
                    Some(hint)
                } else {
                    None
                }
            } else {
                None
            }
        }

        // 子命令提示：显示第一个匹配的子命令作为内联提示
        crate::completion::CompletionContext::Subcommand(ref cmd_name, ref subcmd_prefix) => {
            if let Some(cmd) = completion_manager.get_command(cmd_name) {
                let completions = cmd.get_subcommand_completions(subcmd_prefix);
                if let Some((name, _desc)) = completions.first() {
                    let hint = if subcmd_prefix.is_empty() {
                        name.clone()
                    } else if name.starts_with(subcmd_prefix) {
                        name[subcmd_prefix.len()..].to_string()
                    } else {
                        name.clone()
                    };
                    Some(hint)
                } else {
                    None
                }
            } else {
                None
            }
        }

        _ => None,
    }
}
