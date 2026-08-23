//! 生成提示文本的样式

use colored::*;

/// 标准模式下的提示样式
pub fn std_hint(text: &str) -> String {
    format!("{}", text.dimmed())
}

/// Moe 模式下的提示样式
pub fn moe_hint(text: &str) -> String {
    format!("{}", text.truecolor(255, 182, 193).dimmed())
}

/// 根据模式选择提示样式
pub fn moe_or_std(text: &str, is_moe: bool) -> String {
    if is_moe {
        moe_hint(text)
    } else {
        std_hint(text)
    }
}

/// 参数提示
pub fn arg_hint(name: &str, desc: &str, is_moe: bool) -> String {
    if is_moe {
        format!(
            "  {}  {}",
            name.truecolor(255, 105, 180).bold(),
            desc.truecolor(255, 182, 193).dimmed()
        )
    } else {
        format!(
            "  {}  {}",
            name.bright_blue().bold(),
            desc.dimmed()
        )
    }
}

/// 命令提示
pub fn cmd_hint(name: &str, desc: &str, is_moe: bool) -> String {
    if is_moe {
        format!(
            "  {}  {}",
            name.truecolor(255, 105, 180).bold(),
            desc.truecolor(255, 182, 193).dimmed()
        )
    } else {
        format!(
            "  {}  {}",
            name.bright_green().bold(),
            desc.dimmed()
        )
    }
}
