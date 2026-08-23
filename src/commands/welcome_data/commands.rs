use colored::*;

pub fn push_commands(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{} {}",
            "💖 Commands:".truecolor(255, 160, 122).bold(),
            "💕"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List directory contents {}",
            "ls".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List with detailed information {}",
            "ls -l".truecolor(255, 182, 193).bold(),
            "💖"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List including hidden files {}",
            "ls -a".truecolor(255, 182, 193).bold(),
            "🌸"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List contents of specified directory {}",
            "ls <path>".truecolor(255, 182, 193).bold(),
            "💫"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Print current working directory {}",
            "pwd".truecolor(255, 182, 193).bold(),
            "💖"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Copy current directory path to clipboard {}",
            "cppwd".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Copy file absolute path to clipboard {}",
            "cpf <file>".truecolor(255, 182, 193).bold(),
            "💗"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Change directory {}",
            "cd <path>".truecolor(255, 182, 193).bold(),
            "💕"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Change back to previous directory {}",
            "cd -b/-back".truecolor(255, 182, 193).bold(),
            "🌸"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Jump to directory with .index file matching tag {}",
            "cd -tag <tag>".truecolor(255, 182, 193).bold(),
            "🔖"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Open file with default application {}",
            "open <path>".truecolor(255, 182, 193).bold(),
            "📂"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Move file/folder to destination {}",
            "mv <source> <dest>".truecolor(255, 182, 193).bold(),
            "📦"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Copy file/folder to destination {}",
            "mv <source> <dest> --cp".truecolor(255, 182, 193).bold(),
            "💖"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Create a file {}",
            "mkdf -f <path>".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Create a directory {}",
            "mkdf -d <path>".truecolor(255, 182, 193).bold(),
            "📁"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Manage path aliases {}",
            "alias".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Manage file tags {}",
            "tag".truecolor(255, 182, 193).bold(),
            "💕"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Clear the screen {}",
            "clear".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Show help information {}",
            "help".truecolor(255, 182, 193).bold(),
            "💖"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Exit the program {}",
            "exit".truecolor(255, 182, 193).bold(),
            "👋"
        ));
        out.push('\n');
        out.push('\n');
    } else {
        out.push_str(&format!("{}", "Commands:".bright_yellow().bold()));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List directory contents",
            "ls".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List with detailed information",
            "ls -l".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List including hidden files",
            "ls -a".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - List contents of specified directory",
            "ls <path>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Print current working directory",
            "pwd".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Copy current directory path to clipboard",
            "cppwd".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Copy file absolute path to clipboard",
            "cpf <file>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Change directory",
            "cd <path>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Change back to previous directory",
            "cd -b/-back".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Jump to directory with .index file matching tag",
            "cd -tag <tag>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Open file with default application",
            "open <path>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Move file/folder to destination",
            "mv <source> <dest>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Copy file/folder to destination",
            "mv <source> <dest> --cp".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Create a file",
            "mkdf -f <path>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Create a directory",
            "mkdf -d <path>".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Manage path aliases",
            "alias".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!("  {}  - Manage file tags", "tag".cyan().bold()));
        out.push('\n');
        out.push_str(&format!("  {}  - Clear the screen", "clear".cyan().bold()));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Show help information",
            "help".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!("  {}  - Exit the program", "exit".cyan().bold()));
        out.push('\n');
        out.push('\n');
    }
}
