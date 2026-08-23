use colored::*;

pub fn push_keyboard(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{}\n",
            "⌨️💖 Keyboard Shortcuts~:".truecolor(255, 160, 122).bold()
        ));
        out.push_str(&format!(
            "  {}        Clear current input line in REPL mode ✨\n\n",
            "ESC".truecolor(255, 182, 193).bold()
        ));
    } else {
        out.push_str(&format!(
            "{}\n",
            "⌨️ Keyboard Shortcuts:".bright_yellow().bold()
        ));
        out.push_str(&format!(
            "  {}        Clear current input line in REPL mode\n\n",
            "ESC".cyan().bold()
        ));
    }
}

pub fn push_aliases(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{}\n\n",
            "✨💖 Path Aliases~:".truecolor(255, 105, 180).bold()
        ));
        out.push_str(&format!(
            "  Use {} prefix to use path aliases for faster navigation 💕\n",
            "@".truecolor(255, 160, 122).bold()
        ));
        out.push_str("  Example:\n");
        out.push_str(&format!(
            "    {}              Add alias for project directory ✨\n",
            "alias add proj ~/projects".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}               List directory using alias 💖\n",
            "ls @proj".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}        Navigate to subdirectory using alias 💫\n",
            "cd @proj/rfe".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}                Open file using alias 📂\n",
            "open @proj/rfe/src/main.rs".truecolor(255, 182, 193)
        ));
        out.push_str("  Aliases are saved persistently and available across sessions 💕\n\n");
    } else {
        out.push_str(&format!("{}\n\n", "✨ Path Aliases:".bright_green().bold()));
        out.push_str(&format!(
            "  Use {} prefix to use path aliases for faster navigation\n",
            "@".yellow().bold()
        ));
        out.push_str("  Example:\n");
        out.push_str(&format!(
            "    {}              Add alias for project directory\n",
            "alias add proj ~/projects".cyan()
        ));
        out.push_str(&format!(
            "    {}               List directory using alias\n",
            "ls @proj".cyan()
        ));
        out.push_str(&format!(
            "    {}        Navigate to subdirectory using alias\n",
            "cd @proj/rfe".cyan()
        ));
        out.push_str(&format!(
            "    {}                Open file using alias\n",
            "open @proj/rfe/src/main.rs".cyan()
        ));
        out.push_str("  Aliases are saved persistently and available across sessions\n\n");
    }
}

pub fn push_tags(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{}\n\n",
            "✨💖 File Tags~:".truecolor(255, 105, 180).bold()
        ));
        out.push_str("  Add custom tags to files and directories for better organization 💕\n");
        out.push_str("  Example:\n");
        out.push_str(&format!(
            "    {}              Add tags 'work' and 'rust' to file ✨\n",
            "tag add src/main.rs work rust".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}              Remove tag 'old' from file 💔\n",
            "tag remove src/main.rs old".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}              Get all tags of file 💖\n",
            "tag get src/main.rs".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}              List all files with tags 📋\n",
            "tag list".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}              List files and their tags in current directory ✨\n",
            "ls -tag".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}            List files in current directory tagged 'rust' 🌸\n",
            "ls -t rust".truecolor(255, 182, 193)
        ));
        out.push_str(&format!(
            "    {}  Find files matching both 'rust' and 'doc' tags 🔍\n",
            "tag find rust doc".truecolor(255, 182, 193)
        ));
        out.push_str(
            "  Supports regex matching, multi-tag queries, and automatic backup persistence 💕\n\n",
        );
    } else {
        out.push_str(&format!("{}\n\n", "✨ File Tags:".bright_green().bold()));
        out.push_str("  Add custom tags to files and directories for better organization\n");
        out.push_str("  Example:\n");
        out.push_str(&format!(
            "    {}              Add tags 'work' and 'rust' to file\n",
            "tag add src/main.rs work rust".cyan()
        ));
        out.push_str(&format!(
            "    {}              Remove tag 'old' from file\n",
            "tag remove src/main.rs old".cyan()
        ));
        out.push_str(&format!(
            "    {}              Get all tags of file\n",
            "tag get src/main.rs".cyan()
        ));
        out.push_str(&format!(
            "    {}              List all files with tags\n",
            "tag list".cyan()
        ));
        out.push_str(&format!(
            "    {}              List files and their tags in current directory\n",
            "ls -tag".cyan()
        ));
        out.push_str(&format!(
            "    {}            List files in current directory tagged 'rust'\n",
            "ls -t rust".cyan()
        ));
        out.push_str(&format!(
            "    {}  Find files matching both 'rust' and 'doc' tags\n",
            "tag find rust doc".cyan()
        ));
        out.push_str(
            "  Supports regex matching, multi-tag queries, and automatic backup persistence\n\n",
        );
    }
}

pub fn push_chain(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{}\n\n",
            "✨💖 Command Chain~:".truecolor(255, 105, 180).bold()
        ));
        out.push_str(&format!(
            "  Use {} to chain commands with sequential execution and output passing 💕\n",
            "->".truecolor(255, 160, 122).bold()
        ));
        out.push_str(&format!(
            "  Example: {} pwd -> ls -> cd .. -> pwd ✨\n",
            "$".truecolor(255, 105, 180)
        ));
        out.push_str(&format!(
            "  Use {} to continue execution even if previous command fails 💪\n",
            "->!".truecolor(255, 160, 122).bold()
        ));
        out.push_str(&format!(
            "  Example: {} cd non_exist! -> ls 💫\n",
            "$".truecolor(255, 105, 180)
        ));
        out.push_str("  Use {} as placeholder to insert previous command's output ✨\n");
        out.push_str(&format!(
            "  Example: {} cppwd -> alias add desktop {{}} 💖\n",
            "$".truecolor(255, 105, 180)
        ));
        out.push_str(&format!(
            "  Use {} to pop path levels (each .pop or . goes up one level) 🌸\n",
            "{}.pop.pop...".truecolor(255, 160, 122).bold()
        ));
        out.push_str(&format!(
            "  Example: {} pwd -> cd {{}}.pop.pop  (go up 2 directories) 💫\n",
            "$".truecolor(255, 105, 180)
        ));
        out.push_str("  Shorthand: {}.pop ≡ {}.  /  {}.pop.pop ≡ {}.. ✨\n\n");
    } else {
        out.push_str(&format!(
            "{}\n\n",
            "✨ Command Chain:".bright_green().bold()
        ));
        out.push_str(&format!(
            "  Use {} to chain commands with sequential execution and output passing\n",
            "->".yellow().bold()
        ));
        out.push_str(&format!(
            "  Example: {} pwd -> ls -> cd .. -> pwd\n",
            "$".bright_black()
        ));
        out.push_str(&format!(
            "  Use {} to continue execution even if previous command fails\n",
            "->!".yellow().bold()
        ));
        out.push_str(&format!(
            "  Example: {} cd non_exist! -> ls\n",
            "$".bright_black()
        ));
        out.push_str("  Use {} as placeholder to insert previous command's output\n");
        out.push_str(&format!(
            "  Example: {} cppwd -> alias add desktop {{}}\n",
            "$".bright_black()
        ));
        out.push_str(&format!(
            "  Use {} to pop path levels (each .pop or . goes up one level)\n",
            "{}.pop.pop...".yellow().bold()
        ));
        out.push_str(&format!(
            "  Example: {} pwd -> cd {{}}.pop.pop  (go up 2 directories)\n",
            "$".bright_black()
        ));
        out.push_str("  Shorthand: {}.pop ≡ {}.  /  {}.pop.pop ≡ {}..\n\n");
    }
}
