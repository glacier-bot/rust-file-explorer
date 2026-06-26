//! 高亮显示模块
//! 提供提示符和内容的高亮功能

use colored::*;

/// 高亮显示提示符
pub fn highlight_prompt(prompt: &str) -> String {
    if prompt.starts_with("rfe 🌸 ") && prompt.contains(" 💖 >") {
        let start = "rfe 🌸 ".len();
        let end = prompt.find(" 💖 >").unwrap_or(prompt.len());
        let dir = &prompt[start..end];
        format!(
            "{} {} {} {} {}",
            "rfe".truecolor(255, 105, 180).bold(),
            "🌸".truecolor(255, 182, 193),
            dir.truecolor(255, 182, 193).bold(),
            "💖".truecolor(255, 105, 180),
            ">".truecolor(255, 105, 180).bold()
        )
    } else if prompt.starts_with("rfe ") && prompt.ends_with(" >") {
        let dir = &prompt[4..prompt.len() - 2];
        format!(
            "{} {} {}",
            "rfe".bright_green().bold(),
            dir.bright_blue().bold(),
            ">".bright_blue().bold()
        )
    } else {
        prompt.to_string()
    }
}
