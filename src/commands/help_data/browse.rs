use colored::*;

pub fn push_header(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{} {}\n\n",
            "📖💖 Available Commands~:".truecolor(255, 160, 122).bold(),
            "💕"
        ));
    } else {
        out.push_str(&format!("{}\n\n", "📖 Available Commands:".bright_yellow().bold()));
    }
}

pub fn push_ls(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "  {}               List contents of current directory (with line numbers) ✨\n",
            "ls".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}            List with detailed information 💖\n",
            "ls -l".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}             List including hidden files 🌸\n",
            "ls -a".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}       List contents of specified directory 💫\n",
            "ls <path>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}       List files with their tags 💗\n",
            "ls -tag".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!("  {}      List files matching specified tag regex, supports multi-tag combinations ✨\n", "ls -t/--tag <tag-regex>".truecolor(255, 182, 193).bold()));
        out.push_str(&format!(
            "  {}       Search for files/directories using regex pattern 🔍\n",
            "ls --re <pattern>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}    Search recursively with regex 💫\n",
            "ls --re-deep <pattern>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}  Case-insensitive regex search ✨\n",
            "ls --re --xcaps <pattern>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}  Case-insensitive recursive regex search 💖\n\n",
            "ls --re-deep --xcaps <pattern>"
                .truecolor(255, 182, 193)
                .bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}               List contents of current directory (with line numbers)\n",
            "ls".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}            List with detailed information\n",
            "ls -l".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}             List including hidden files\n",
            "ls -a".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}       List contents of specified directory\n",
            "ls <path>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}       List files with their tags\n",
            "ls -tag".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}      List files matching specified tag regex, supports multi-tag combinations\n",
            "ls -t/--tag <tag-regex>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}       Search for files/directories using regex pattern\n",
            "ls --re <pattern>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}    Search recursively with regex\n",
            "ls --re-deep <pattern>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}  Case-insensitive regex search\n",
            "ls --re --xcaps <pattern>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}  Case-insensitive recursive regex search\n\n",
            "ls --re-deep --xcaps <pattern>".cyan().bold()
        ));
    }
}

pub fn push_regex(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "{}\n",
            "📝💖 Common Regex Syntax~:".truecolor(255, 160, 122).bold()
        ));
        out.push_str(&format!("  {}  Match any single character                e.g. ls --re fi.e  =>  file, fine ✨\n", ".".truecolor(255, 192, 203)));
        out.push_str(&format!(
            "  {}  Match previous char 0+ times              e.g. ls --re a*   =>  a, aa, aaa 💫\n",
            "*".truecolor(255, 192, 203)
        ));
        out.push_str(&format!(
            "  {}  Match previous char 1+ times              e.g. ls --re a+   =>  a, aa, aaa ✨\n",
            "+".truecolor(255, 192, 203)
        ));
        out.push_str(&format!("  {}  Match previous char 0 or 1 time           e.g. ls --re colou?r  =>  color, colour 💖\n", "?".truecolor(255, 192, 203)));
        out.push_str(&format!("  {}  Match start of string                     e.g. ls --re ^src  =>  files starting with src 🌸\n", "^".truecolor(255, 192, 203)));
        out.push_str(&format!("  {}  Match end of string                       e.g. ls --re \\.rs$  =>  all .rs files 💫\n", "$".truecolor(255, 192, 203)));
        out.push_str(&format!("  {}  Match any char in set                     e.g. ls --re [Ff]ile  =>  File, file ✨\n", "[abc]".truecolor(255, 192, 203)));
        out.push_str(&format!("  {}  Match any char NOT in set                 e.g. ls --re [^Ff]ile  =>  aile, bile... 💖\n", "[^abc]".truecolor(255, 192, 203)));
        out.push_str(&format!("  {}  OR logic, match either expression         e.g. ls --re \\.rs$|\\.toml$  =>  .rs and .toml files 🌸\n", "|".truecolor(255, 192, 203)));
        out.push_str(&format!("  {}  Grouping for combining expressions         e.g. ls --re (src|target)\\/  =>  files under src or target 💫\n\n", "()".truecolor(255, 192, 203)));
    } else {
        out.push_str(&format!(
            "{}\n",
            "📝 Common Regex Syntax:".bright_yellow().bold()
        ));
        out.push_str(&format!(
            "  {}  Match any single character                e.g. ls --re fi.e  =>  file, fine\n",
            ".".bright_cyan()
        ));
        out.push_str(&format!(
            "  {}  Match previous char 0+ times              e.g. ls --re a*   =>  a, aa, aaa\n",
            "*".bright_cyan()
        ));
        out.push_str(&format!(
            "  {}  Match previous char 1+ times              e.g. ls --re a+   =>  a, aa, aaa\n",
            "+".bright_cyan()
        ));
        out.push_str(&format!("  {}  Match previous char 0 or 1 time           e.g. ls --re colou?r  =>  color, colour\n", "?".bright_cyan()));
        out.push_str(&format!("  {}  Match start of string                     e.g. ls --re ^src  =>  files starting with src\n", "^".bright_cyan()));
        out.push_str(&format!("  {}  Match end of string                       e.g. ls --re \\.rs$  =>  all .rs files\n", "$".bright_cyan()));
        out.push_str(&format!("  {}  Match any char in set                     e.g. ls --re [Ff]ile  =>  File, file\n", "[abc]".bright_cyan()));
        out.push_str(&format!("  {}  Match any char NOT in set                 e.g. ls --re [^Ff]ile  =>  aile, bile...\n", "[^abc]".bright_cyan()));
        out.push_str(&format!("  {}  OR logic, match either expression         e.g. ls --re \\.rs$|\\.toml$  =>  .rs and .toml files\n", "|".bright_cyan()));
        out.push_str(&format!("  {}  Grouping for combining expressions         e.g. ls --re (src|target)\\/  =>  files under src or target\n\n", "()".bright_cyan()));
    }
}

pub fn push_pwd(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "  {}              Print current working directory 💖\n",
            "pwd".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}   Copy current directory path to clipboard ✨\n",
            "cppwd".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}  Copy file absolute path to clipboard 💗\n",
            "cpf <file>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}       Copy path by ls line number ✨\n\n",
            "cpf -r <line>".truecolor(255, 182, 193).bold()
        ));
    } else {
        out.push_str(&format!(
            "  {}              Print current working directory\n",
            "pwd".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}   Copy current directory path to clipboard\n",
            "cppwd".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}  Copy file absolute path to clipboard\n",
            "cpf <file>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}       Copy path by ls line number\n\n",
            "cpf -r <line>".cyan().bold()
        ));
    }
}

pub fn push_cd(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!(
            "  {}            Change to home directory 🏠\n",
            "cd".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}         Change to parent directory ⬆️\n",
            "cd ..".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}     Change to specified directory 💫\n",
            "cd <path>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}  Change to directory by ls line number ✨\n",
            "cd -r <line>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}     Change back to previous directory 🔙\n",
            "cd -b/-back".truecolor(255, 182, 193).bold()
        ));
        out.push_str("                      (short: -b, long: -back) 💕\n");
        out.push_str(&format!(
            "  {}   Jump to directory with .index file matching tag 🔖\n",
            "cd -tag <tag>".truecolor(255, 182, 193).bold()
        ));
        out.push_str("                      supports regex matching 💕\n\n");
    } else {
        out.push_str(&format!(
            "  {}            Change to home directory\n",
            "cd".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}         Change to parent directory\n",
            "cd ..".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}     Change to specified directory\n",
            "cd <path>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}  Change to directory by ls line number\n",
            "cd -r <line>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}     Change back to previous directory\n",
            "cd -b/-back".cyan().bold()
        ));
        out.push_str("                      (short: -b, long: -back)\n");
        out.push_str(&format!(
            "  {}   Jump to directory with .index file matching tag\n",
            "cd -tag <tag>".cyan().bold()
        ));
        out.push_str("                      supports regex matching\n\n");
    }
}

pub fn push_open(out: &mut String, moe: bool) {
    if moe {
        out.push_str(&format!("  {}         Open file with default application / Open directory in file explorer 📂\n", "open <path>".truecolor(255, 182, 193).bold()));
        out.push_str(&format!(
            "  {}       Open file/directory by ls line number ✨\n",
            "open -r <line>".truecolor(255, 182, 193).bold()
        ));
        out.push_str(&format!(
            "  {}   Open directory with .index file matching tag 🔖\n",
            "open -tag <tag>".truecolor(255, 182, 193).bold()
        ));
        out.push_str("                      supports regex matching 💕\n\n");
    } else {
        out.push_str(&format!(
            "  {}         Open file with default application / Open directory in file explorer\n",
            "open <path>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}       Open file/directory by ls line number\n",
            "open -r <line>".cyan().bold()
        ));
        out.push_str(&format!(
            "  {}   Open directory with .index file matching tag\n",
            "open -tag <tag>".cyan().bold()
        ));
        out.push_str("                      supports regex matching\n\n");
    }
}
