use colored::*;

pub fn push_mv(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "  {}    Move file/folder to destination 📦\n",
            "mv <source> <dest>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}  Move file/folder by ls line number(s) ✨\n",
            "mv -r <line> <dest> | mv <source> -r <line> | mv -r <line1> -r <line2>"
                .truecolor(255, 182, 193)
                .bold()
        ));
        out.push_str(&format!(
            "  {}    Copy file/folder to destination (preserves original) 💖\n\n",
            "mv <source> <dest> --cp".truecolor(255, 182, 193).bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}    Move file/folder to destination\n",
            "mv <source> <dest>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}  Move file/folder by ls line number(s)\n",
            "mv -r <line> <dest> | mv <source> -r <line> | mv -r <line1> -r <line2>"
                .cyan()
                .bold()
        ));
        out.push_str(&format!(
            "  {}    Copy file/folder to destination (preserves original)\n\n",
            "mv <source> <dest> --cp".cyan().bold()
        ));
    }
}

pub fn push_mkdf(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "  {}    Create a file (auto-creates parent directories) ✨\n",
            "mkdf -f <path>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}      Create a directory 📁\n",
            "mkdf -d <path>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}   Create a directory with parents 🌸\n",
            "mkdf -d -p <path>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}     Show mkdf command help 💖\n\n",
            "mkdf -h/--help".truecolor(255, 182, 193).bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}    Create a file (auto-creates parent directories)\n",
            "mkdf -f <path>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}      Create a directory\n",
            "mkdf -d <path>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}   Create a directory with parents\n",
            "mkdf -d -p <path>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}     Show mkdf command help\n\n",
            "mkdf -h/--help".cyan().bold()
        ));
    }
}

pub fn push_change(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "  {}  Switch to standard mode ✨\n",
            "change -std".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}  Switch to moe moe mode 💕\n\n",
            "change -moe".truecolor(255, 182, 193).bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}  Switch to standard mode\n",
            "change -std".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}  Switch to moe moe mode\n\n",
            "change -moe".cyan().bold()
        ));
    }
}

pub fn push_misc(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "  {}             Exit the program 👋\n",
            "exit".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}            Clear the screen ✨\n",
            "clear".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}             Show this help 💖\n",
            "help".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}            Manage path aliases ✨\n\n",
            "alias".truecolor(255, 182, 193).bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}             Exit the program\n",
            "exit".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}            Clear the screen\n",
            "clear".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}             Show this help\n",
            "help".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}            Manage path aliases\n\n",
            "alias".cyan().bold()
        ));
    }
}

pub fn push_shell(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{}\n\n",
            "🐚💖 Shell Integration~:".truecolor(255, 105, 180).bold()
        ));
        out.push_str("  Any unrecognized command is automatically passed to the system shell 💕\n");
        out.push_str(&format!(
            "  Example: {} echo Hello World  =>  Execute shell command ✨\n",
            "echo Hello World".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "  Example: {}             =>  List files with ls command 💖\n",
            "dir".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "  Example: {}     =>  Change directory (synced with rfe) 💫\n",
            "cd <path>".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "  Shell command output can be used with command chaining {} ✨\n\n",
            "->".truecolor(255, 160, 122).bold()
        ));
    } else {
        out.push_str(&format!("{}\n\n", "🐚 Shell Integration:".bright_green().bold()));
        out.push_str("  Any unrecognized command is automatically passed to the system shell\n");
        out.push_str(&format!(
            "  Example: {} echo Hello World  =>  Execute shell command\n",
            "echo Hello World".cyan()
        ));
        out.push_str(&format!(
            "  Example: {}             =>  List files with ls command\n",
            "dir".cyan()
        ));
        out.push_str(&format!(
            "  Example: {}     =>  Change directory (synced with rfe)\n",
            "cd <path>".cyan()
        ));
        out.push_str(&format!(
            "  Shell command output can be used with command chaining {}\n\n",
            "->".yellow().bold()
        ));
    }
}
