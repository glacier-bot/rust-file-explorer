use colored::*;
use std::env;

pub fn cmd_pwd() -> Result<(String, String), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let plain_path = current_dir.display().to_string();
    let display_output = plain_path.bright_cyan().to_string();
    Ok((display_output, plain_path))
}