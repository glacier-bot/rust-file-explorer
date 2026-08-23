use colored::*;

pub fn push_features(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{} {}",
            "💖 Powerful features:".truecolor(255, 160, 122).bold(),
            "💕"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Chain commands, pass output to next {}",
            "cmd -> cmd".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Placeholder for previous command output {}",
            "{}".truecolor(255, 182, 193).bold(),
            "💫"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Pop path (each .pop or . goes up one directory level) {}",
            "{}.pop.pop...".truecolor(255, 182, 193).bold(),
            "💖"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Use path alias (@alias) {}",
            "@<alias>".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
        out.push('\n');
    } else {
        out.push_str(&format!("{}", "✨ Powerful features:".bright_blue().bold()));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Chain commands, pass previous output to next",
            "cmd -> cmd".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Placeholder for previous command output",
            "{}".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Pop path (each .pop or . goes up one directory level)",
            "{}.pop.pop...".cyan().bold()
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Use path alias (@alias)",
            "@<alias>".cyan().bold()
        ));
        out.push('\n');
        out.push('\n');
    }
}

pub fn push_keyboard(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{} {}",
            "💖 Keyboard shortcuts:".truecolor(255, 160, 122).bold(),
            "💕"
        ));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Clear current input line in REPL mode {}",
            "ESC".truecolor(255, 182, 193).bold(),
            "✨"
        ));
        out.push('\n');
    } else {
        out.push_str(&format!("{}", "Keyboard shortcuts:".bright_yellow().bold()));
        out.push('\n');
        out.push_str(&format!(
            "  {}  - Clear current input line in REPL mode",
            "ESC".cyan().bold()
        ));
        out.push('\n');
    }
}
