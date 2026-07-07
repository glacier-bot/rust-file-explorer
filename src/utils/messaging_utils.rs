
//! 消息格式化工具模块
//! 提供通用的消息格式化函数，减少代码冗余

use colored::{Color, ColoredString, Colorize};

/// Moe 模式颜色常量
#[allow(dead_code)]
pub const MOE_PINK: Color = Color::TrueColor { r: 255, g: 105, b: 180 };
#[allow(dead_code)]
pub const MOE_LIGHT_PINK: Color = Color::TrueColor { r: 255, g: 182, b: 193 };
#[allow(dead_code)]
pub const MOE_SALMON: Color = Color::TrueColor { r: 255, g: 160, b: 122 };

/// 判断当前是否为 moe 模式，并执行对应的格式化函数
///
/// # Arguments
/// * `moe_fn` - moe 模式下的格式化函数
/// * `std_fn` - 标准模式下的格式化函数
///
/// # Returns
/// * 格式化后的字符串
pub fn format_by_mode<F, G>(moe_fn: F, std_fn: G) -> String
where
    F: FnOnce() -> String,
    G: FnOnce() -> String,
{
    if crate::utils::moe::is_moe() {
        moe_fn()
    } else {
        std_fn()
    }
}

/// 打印根据模式选择的消息
///
/// # Arguments
/// * `moe_msg` - moe 模式下的消息
/// * `std_msg` - 标准模式下的消息
#[allow(dead_code)]
pub fn print_by_mode(moe_msg: &str, std_msg: &str) {
    if crate::utils::moe::is_moe() {
        println!("{}", moe_msg);
    } else {
        println!("{}", std_msg);
    }
}

/// 格式化错误消息（moe 模式）
pub fn format_moe_error(msg: &str) -> String {
    format!(
        "{} {} {}",
        "😢💔".truecolor(255, 105, 180),
        "Error:".truecolor(255, 105, 180),
        msg.truecolor(255, 182, 193)
    )
}

/// 格式化错误消息（标准模式）
pub fn format_std_error(msg: &str) -> String {
    format!("{} {}", "❌ Error:".red(), msg.bright_red())
}

/// 格式化通用错误消息
pub fn format_error_msg(msg: &str) -> String {
    format_by_mode(|| format_moe_error(msg), || format_std_error(msg))
}

/// 打印错误消息
pub fn print_error_msg(msg: &str) {
    if crate::utils::moe::is_moe() {
        eprintln!("{}", format_moe_error(msg));
    } else {
        eprintln!("{}", format_std_error(msg));
    }
}

/// 格式化命令未找到消息
pub fn format_cmd_not_found(cmd: &str, is_cli: bool) -> String {
    let help_hint = if is_cli {
        "Type 'rfe help' for available commands"
    } else {
        "Type 'help' for available commands"
    };

    if crate::utils::moe::is_moe() {
        format!(
            "{} Command not found: {}. {}~ 💕",
            "😢".truecolor(255, 105, 180),
            cmd.truecolor(255, 182, 193),
            help_hint
        )
    } else {
        format!(
            "{} Command not found: {}. {}.",
            "❌".red(),
            cmd.cyan(),
            help_hint
        )
    }
}

/// 格式化退出消息（moe 模式）
pub fn format_moe_exit() -> String {
    "👋🌸💖 Bye-bye! See you next time~ 💕"
        .truecolor(255, 182, 193)
        .to_string()
}

/// 格式化退出消息（标准模式）
pub fn format_std_exit() -> String {
    "👋 Goodbye!".bright_green().to_string()
}

/// 格式化路径边界警告（moe 模式）
pub fn format_moe_path_boundary(actual_pops: usize, previous_data: &str) -> String {
    format!(
        "{} {} {} {}",
        "✨".truecolor(255, 182, 193),
        "Oopsie!".truecolor(255, 105, 180).bold(),
        "Can't go any higher, nya~ 💕".truecolor(255, 182, 193),
        format!("(Stopped after {} pop(s) from '{}' )", actual_pops, previous_data)
            .truecolor(255, 182, 193)
    )
}

/// 格式化路径边界警告（标准模式）
pub fn format_std_path_boundary(actual_pops: usize, previous_data: &str) -> String {
    format!(
        "{} {} {}",
        "⚠".yellow().bold(),
        "Path boundary reached:".yellow().bold(),
        format!("stopped after {} pop(s) from '{}'", actual_pops, previous_data).yellow()
    )
}

/// 格式化目录变更消息
pub fn format_changed_to(path: &str) -> String {
    format!("{} {}", "Changed to:".green(), path.cyan())
}

/// 格式化 .index 文件缺失警告（moe 模式）
pub fn format_moe_missing_index(dir_path: &str) -> String {
    format!(
        "{} {} {} {}",
        "✨".truecolor(255, 182, 193),
        "Warning!".truecolor(255, 105, 180).bold(),
        "Missing .index file in folder nya~ 💕".truecolor(255, 182, 193),
        format!("({})", dir_path).truecolor(255, 160, 122)
    )
}

/// 格式化 .index 文件缺失警告（标准模式）
pub fn format_std_missing_index(dir_path: &str) -> String {
    format!(
        "{} {} {}",
        "⚠".yellow().bold(),
        "Warning: Missing .index file in folder".yellow().bold(),
        format!("({})", dir_path).yellow()
    )
}

/// 获取 moe 模式下的彩色字符串
#[allow(dead_code)]
pub fn moe_style(text: &str, color: Color) -> ColoredString {
    text.truecolor(
        match color {
            Color::TrueColor { r, .. } => r,
            _ => 255,
        },
        match color {
            Color::TrueColor { g, .. } => g,
            _ => 182,
        },
        match color {
            Color::TrueColor { b, .. } => b,
            _ => 193,
        },
    )
}
