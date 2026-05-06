use colored::*;
use std::env;
use crate::utils::moe::is_moe;

pub fn cmd_pwd() -> Result<(String, String), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let plain_path = current_dir.display().to_string();
    let display_output = if is_moe() {
        format!("{} {}", "💖📂".truecolor(255, 105, 180), plain_path.truecolor(255, 182, 193))
    } else {
        plain_path.bright_cyan().to_string()
    };
    Ok((display_output, plain_path))
}