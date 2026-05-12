use crate::utils::moe::{disable_moe, enable_moe, is_moe};
use colored::*;

pub fn cmd_change(args: &[&str]) -> Result<(String, String), Box<dyn std::error::Error>> {
    if args.is_empty() {
        return Err("Usage: change -std (standard mode) or change -moe (moe mode)".into());
    }

    match args[0] {
        "-std" | "--std" => {
            if !is_moe() {
                let output = if is_moe() {
                    "Already in standard mode 💕".to_string()
                } else {
                    format!("{} Already in standard mode ✨", "✓".bright_green())
                };
                return Ok((output, String::new()));
            }
            disable_moe();
            let output = if is_moe() {
                "Failed to switch to standard mode 😢💔".to_string()
            } else {
                format!("{} Switched to standard mode ✨", "✓".bright_green())
            };
            Ok((output, String::new()))
        }
        "-moe" | "--moe" => {
            if is_moe() {
                let output = if is_moe() {
                    format!(
                        "{} {} Already in moe moe mode! 💕✨🌸",
                        "✓".truecolor(255, 105, 180),
                        "💖"
                    )
                } else {
                    "Already in moe mode ✨".to_string()
                };
                return Ok((output, String::new()));
            }
            enable_moe();
            let output = if is_moe() {
                format!(
                    "{} {} Switched to moe moe mode! 💕✨🌸",
                    "✓".truecolor(255, 105, 180),
                    "💖"
                )
            } else {
                "Failed to switch to moe moe mode 😢💔".to_string()
            };
            Ok((output, String::new()))
        }
        _ => Err("Usage: change -std (standard mode) or change -moe (moe mode)".into()),
    }
}
