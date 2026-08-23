use crate::commands::welcome_data::{banner, commands, extras};
use crate::utils::moe::is_moe;
use colored::*;
use std::time::Instant;

pub fn cmd_welcome() -> Result<(String, String), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let output = if is_moe() {
        format_moe_welcome()
    } else {
        format_standard_welcome()
    };

    let elapsed = start.elapsed();
    if elapsed.as_millis() > 300 {
        eprintln!(
            "{} Welcome page load time: {}ms (exceeds 300ms limit)",
            "⚠️ Warning:".yellow(),
            elapsed.as_millis()
        );
    }

    Ok((output, String::new()))
}

fn format_standard_welcome() -> String {
    let mut output = String::new();
    banner::push_banner(&mut output, false);
    commands::push_commands(&mut output, false);
    extras::push_features(&mut output, false);
    extras::push_keyboard(&mut output, false);
    output
}

fn format_moe_welcome() -> String {
    let mut output = String::new();
    banner::push_banner(&mut output, true);
    commands::push_commands(&mut output, true);
    extras::push_features(&mut output, true);
    extras::push_keyboard(&mut output, true);
    output
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::moe::enable_moe;
    // use std::sync::atomic::Ordering;

    #[test]
    fn test_welcome_standard_mode() {
        let result = cmd_welcome();
        assert!(result.is_ok());
        let (display, raw) = result.unwrap();
        assert!(!display.is_empty());
        assert!(display.contains("Rust File Explorer"));
        assert!(display.contains("Commands:"));
        assert!(raw.is_empty());
    }

    #[test]
    fn test_welcome_moe_mode() {
        enable_moe();
        let result = cmd_welcome();
        assert!(result.is_ok());
        let (display, raw) = result.unwrap();
        assert!(!display.is_empty());
        assert!(display.contains("moe moe mode"));
        assert!(display.contains("🌸"));
        assert!(raw.is_empty());
    }

    #[test]
    fn test_welcome_performance() {
        let start = Instant::now();
        let result = cmd_welcome();
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed.as_millis() <= 300,
            "Welcome page load time {}ms exceeds 300ms limit",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_welcome_mode_switch() {
        let result1 = cmd_welcome();
        assert!(result1.is_ok());
        let (display1, _) = result1.unwrap();

        let is_moe_before = is_moe();

        enable_moe();
        let result2 = cmd_welcome();
        assert!(result2.is_ok());
        let (display2, _) = result2.unwrap();

        if !is_moe_before {
            assert_ne!(
                display1, display2,
                "Standard and moe mode displays should differ"
            );
        }
    }

    #[test]
    fn test_welcome_content_completeness() {
        let result = cmd_welcome();
        assert!(result.is_ok());
        let (display, _) = result.unwrap();

        assert!(display.contains("ls"));
        assert!(display.contains("pwd"));
        assert!(display.contains("cd"));
        assert!(display.contains("cd -tag"));
        assert!(display.contains("exit"));
        assert!(display.contains("help"));
        assert!(display.contains("Keyboard shortcuts"));
    }

    #[test]
    fn test_welcome_no_truncation() {
        let result = cmd_welcome();
        assert!(result.is_ok());
        let (display, _) = result.unwrap();

        let lines: Vec<&str> = display.lines().collect();
        for line in &lines {
            let visible_len = strip_ansi_escapes(line).chars().count();
            assert!(
                visible_len <= 80,
                "Line exceeds terminal width ({} chars): {}",
                visible_len,
                line
            );
        }
    }

    fn strip_ansi_escapes(input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}
