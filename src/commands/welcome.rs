use crate::utils::moe::is_moe;
use crate::utils::version::VERSION;
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

    output.push_str(&format!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".bright_green()
    ));
    output.push('\n');
    output.push_str(&format!(
        "{}",
        format!(
            "║           Rust File Explorer v{}                         ║",
            VERSION
        )
        .bright_green()
    ));
    output.push('\n');
    output.push_str(&format!(
        "{}",
        "║           A cross-platform CLI file browser                  ║".bright_green()
    ));
    output.push('\n');
    output.push_str(&format!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".bright_green()
    ));
    output.push('\n');
    output.push('\n');
    output.push_str(&format!("{}", "Commands:".bright_yellow().bold()));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List directory contents",
        "ls".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List with detailed information",
        "ls -l".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List including hidden files",
        "ls -a".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List contents of specified directory",
        "ls <path>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Print current working directory",
        "pwd".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Copy current directory path to clipboard",
        "cppwd".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Copy file absolute path to clipboard",
        "cpf <file>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Change directory",
        "cd <path>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Change back to previous directory",
        "cd -b/-back".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Jump to directory with .index file matching tag",
        "cd -idx <tag>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Open file with default application",
        "open <path>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Move file/folder to destination",
        "mv <source> <dest>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Copy file/folder to destination",
        "mv <source> <dest> --cp".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Create a file",
        "mkdf -f <path>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Create a directory",
        "mkdf -d <path>".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Manage path aliases",
        "alias".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!("  {}  - Manage file tags", "tag".cyan().bold()));
    output.push('\n');
    output.push_str(&format!("  {}  - Clear the screen", "clear".cyan().bold()));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Show help information",
        "help".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!("  {}  - Exit the program", "exit".cyan().bold()));
    output.push('\n');
    output.push('\n');
    output.push_str(&format!("{}", "✨ Powerful features:".bright_blue().bold()));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Chain commands, pass previous output to next",
        "cmd -> cmd".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Placeholder for previous command output",
        "{}".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Pop path (each .pop or . goes up one directory level)",
        "{}.pop.pop...".cyan().bold()
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Use path alias (@alias)",
        "@<alias>".cyan().bold()
    ));
    output.push('\n');
    output.push('\n');
    output.push_str(&format!("{}", "Keyboard shortcuts:".bright_yellow().bold()));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Clear current input line in REPL mode",
        "ESC".cyan().bold()
    ));
    output.push('\n');

    output
}

fn format_moe_welcome() -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".truecolor(255, 105, 180)
    ));
    output.push('\n');
    output.push_str(&format!(
        "{}{}{}",
        "║ ".truecolor(255, 105, 180),
        format!(
            "        🌸✨ Rust File Explorer v{} ✨🌸                ",
            VERSION
        )
        .truecolor(255, 182, 193),
        " ║".truecolor(255, 105, 180)
    ));
    output.push('\n');
    output.push_str(&format!(
        "{}{}{}",
        "║ ".truecolor(255, 105, 180),
        "     ciallo∠・ω⌒☆ Welcome to the moe moe mode！💕           ".truecolor(255, 182, 193),
        " ║".truecolor(255, 105, 180)
    ));
    output.push('\n');
    output.push_str(&format!(
        "{}{}{}",
        "║ ".truecolor(255, 105, 180),
        "         A cross-platform CLI file browser 💕               ".truecolor(255, 182, 193),
        " ║".truecolor(255, 105, 180)
    ));
    output.push('\n');
    output.push_str(&format!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".truecolor(255, 105, 180)
    ));
    output.push('\n');
    output.push('\n');
    output.push_str(&format!(
        "{} {}",
        "💖 Commands:".truecolor(255, 160, 122).bold(),
        "💕"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List directory contents {}",
        "ls".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List with detailed information {}",
        "ls -l".truecolor(255, 182, 193).bold(),
        "💖"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List including hidden files {}",
        "ls -a".truecolor(255, 182, 193).bold(),
        "🌸"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - List contents of specified directory {}",
        "ls <path>".truecolor(255, 182, 193).bold(),
        "💫"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Print current working directory {}",
        "pwd".truecolor(255, 182, 193).bold(),
        "💖"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Copy current directory path to clipboard {}",
        "cppwd".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Copy file absolute path to clipboard {}",
        "cpf <file>".truecolor(255, 182, 193).bold(),
        "💗"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Change directory {}",
        "cd <path>".truecolor(255, 182, 193).bold(),
        "💕"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Change back to previous directory {}",
        "cd -b/-back".truecolor(255, 182, 193).bold(),
        "🌸"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Jump to directory with .index file matching tag {}",
        "cd -idx <tag>".truecolor(255, 182, 193).bold(),
        "🔖"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Open file with default application {}",
        "open <path>".truecolor(255, 182, 193).bold(),
        "📂"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Move file/folder to destination {}",
        "mv <source> <dest>".truecolor(255, 182, 193).bold(),
        "📦"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Copy file/folder to destination {}",
        "mv <source> <dest> --cp".truecolor(255, 182, 193).bold(),
        "💖"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Create a file {}",
        "mkdf -f <path>".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Create a directory {}",
        "mkdf -d <path>".truecolor(255, 182, 193).bold(),
        "📁"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Manage path aliases {}",
        "alias".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Manage file tags {}",
        "tag".truecolor(255, 182, 193).bold(),
        "💕"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Clear the screen {}",
        "clear".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Show help information {}",
        "help".truecolor(255, 182, 193).bold(),
        "💖"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Exit the program {}",
        "exit".truecolor(255, 182, 193).bold(),
        "👋"
    ));
    output.push('\n');
    output.push('\n');
    output.push_str(&format!(
        "{} {}",
        "💖 Powerful features:".truecolor(255, 160, 122).bold(),
        "💕"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Chain commands, pass output to next {}",
        "cmd -> cmd".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Placeholder for previous command output {}",
        "{}".truecolor(255, 182, 193).bold(),
        "💫"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Pop path (each .pop or . goes up one directory level) {}",
        "{}.pop.pop...".truecolor(255, 182, 193).bold(),
        "💖"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Use path alias (@alias) {}",
        "@<alias>".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');
    output.push('\n');
    output.push_str(&format!(
        "{} {}",
        "💖 Keyboard shortcuts:".truecolor(255, 160, 122).bold(),
        "💕"
    ));
    output.push('\n');
    output.push_str(&format!(
        "  {}  - Clear current input line in REPL mode {}",
        "ESC".truecolor(255, 182, 193).bold(),
        "✨"
    ));
    output.push('\n');

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
        assert!(display.contains("cd -idx"));
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
