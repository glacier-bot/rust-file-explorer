use crate::utils::version::VERSION;
use colored::*;

pub fn push_banner(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{}",
            "╔══════════════════════════════════════════════════════════════╗".truecolor(255, 105, 180)
        ));
        out.push('\n');
        out.push_str(&format!(
            "{}{}{}",
            "║ ".truecolor(255, 105, 180),
            format!(
                "        🌸✨ Rust File Explorer v{} ✨🌸                ",
                VERSION
            )
            .truecolor(255, 182, 193),
            " ║".truecolor(255, 105, 180)
        ));
        out.push('\n');
        out.push_str(&format!(
            "{}{}{}",
            "║ ".truecolor(255, 105, 180),
            "     ciallo∠・ω⌒☆ Welcome to the moe moe mode！💕           ".truecolor(255, 182, 193),
            " ║".truecolor(255, 105, 180)
        ));
        out.push('\n');
        out.push_str(&format!(
            "{}{}{}",
            "║ ".truecolor(255, 105, 180),
            "         A cross-platform CLI file browser 💕               ".truecolor(255, 182, 193),
            " ║".truecolor(255, 105, 180)
        ));
        out.push('\n');
        out.push_str(&format!(
            "{}",
            "╚══════════════════════════════════════════════════════════════╝".truecolor(255, 105, 180)
        ));
        out.push('\n');
        out.push('\n');
    } else {
        out.push_str(&format!(
            "{}",
            "╔══════════════════════════════════════════════════════════════╗".bright_green()
        ));
        out.push('\n');
        out.push_str(&format!(
            "{}",
            format!(
                "║           Rust File Explorer v{}                         ║",
                VERSION
            )
            .bright_green()
        ));
        out.push('\n');
        out.push_str(&format!(
            "{}",
            "║           A cross-platform CLI file browser                  ║".bright_green()
        ));
        out.push('\n');
        out.push_str(&format!(
            "{}",
            "╚══════════════════════════════════════════════════════════════╝".bright_green()
        ));
        out.push('\n');
        out.push('\n');
    }
}
