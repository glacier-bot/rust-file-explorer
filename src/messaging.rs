//! 消息格式化模块
//! 负责所有用户可见的消息格式化输出，包括错误消息、欢迎信息等
//! 支持普通模式和 moe 模式两种风格

use crate::commands;
use crate::utils::messaging_utils;
use colored::Colorize;

/// 打印欢迎信息
pub fn print_welcome() {
    let (display, _) = commands::welcome::cmd_welcome().unwrap_or_default();
    println!("{}", display);
}

/// 打印错误消息
pub fn print_error(msg: &str) {
    messaging_utils::print_error_msg(msg);
}

/// 格式化带颜色的普通错误消息（返回 String）
pub fn format_error(msg: &str) -> String {
    messaging_utils::format_error_msg(msg)
}

/// 打印退出消息
pub fn print_exit_message() {
    println!("{}", format_exit_message());
}

/// 格式化退出消息
pub fn format_exit_message() -> String {
    messaging_utils::format_by_mode(
        messaging_utils::format_moe_exit,
        messaging_utils::format_std_exit,
    )
}

/// 格式化命令未找到消息（REPL 模式）
pub fn format_command_not_found_repl(cmd: &str) -> String {
    messaging_utils::format_cmd_not_found(cmd, false)
}

/// 格式化命令未找到消息（CLI 模式）
pub fn format_command_not_found_cli(cmd: &str) -> String {
    messaging_utils::format_cmd_not_found(cmd, true)
}

/// 打印路径边界警告
pub fn print_path_boundary_warning(actual_pops: usize, previous_data: &str) {
    println!(
        "{}",
        messaging_utils::format_by_mode(
            || messaging_utils::format_moe_path_boundary(actual_pops, previous_data),
            || messaging_utils::format_std_path_boundary(actual_pops, previous_data),
        )
    );
}

/// 打印 cd selection 提示
pub fn print_cd_selection_prompt() {
    print!("{} Enter selection number: ", "📍".bright_blue());
}

/// 打印 cd selection 取消消息
pub fn print_selection_cancelled() {
    println!("\nSelection cancelled.");
}

/// 格式化目录变更消息
pub fn format_changed_to(path: &str) -> String {
    messaging_utils::format_changed_to(path)
}
