use colored::*;

pub fn cmd_help() -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut output = format!("{}\n\n", "📖 Available Commands:".bright_yellow().bold());
    output.push_str(&format!("  {}               List contents of current directory\n", "ls".cyan().bold()));
    output.push_str(&format!("  {}            List with detailed information\n", "ls -l".cyan().bold()));
    output.push_str(&format!("  {}             List including hidden files\n", "ls -a".cyan().bold()));
    output.push_str(&format!("  {}       List contents of specified directory\n", "ls <path>".cyan().bold()));
    output.push_str(&format!("  {}       List files with their tags\n", "ls -tag".cyan().bold()));
    output.push_str(&format!("  {}      List files matching specified tag regex, supports multi-tag combinations\n", "ls -t/--tag <tag-regex>".cyan().bold()));
    output.push_str(&format!("  {}       Search for files/directories using regex pattern\n", "ls --re <pattern>".cyan().bold()));
    output.push_str(&format!("  {}    Search recursively with regex\n", "ls --re-deep <pattern>".cyan().bold()));
    output.push_str(&format!("  {}  Case-insensitive regex search\n", "ls --re --xcaps <pattern>".cyan().bold()));
    output.push_str(&format!("  {}  Case-insensitive recursive regex search\n\n", "ls --re-deep --xcaps <pattern>".cyan().bold()));

    output.push_str(&format!("{}\n", "📝 Common Regex Syntax:".bright_yellow().bold()));
    output.push_str(&format!("  {}  Match any single character                e.g. ls --re fi.e  =>  file, fine\n", ".".bright_cyan()));
    output.push_str(&format!("  {}  Match previous char 0+ times              e.g. ls --re a*   =>  a, aa, aaa\n", "*".bright_cyan()));
    output.push_str(&format!("  {}  Match previous char 1+ times              e.g. ls --re a+   =>  a, aa, aaa\n", "+".bright_cyan()));
    output.push_str(&format!("  {}  Match previous char 0 or 1 time           e.g. ls --re colou?r  =>  color, colour\n", "?".bright_cyan()));
    output.push_str(&format!("  {}  Match start of string                     e.g. ls --re ^src  =>  files starting with src\n", "^".bright_cyan()));
    output.push_str(&format!("  {}  Match end of string                       e.g. ls --re \\.rs$  =>  all .rs files\n", "$".bright_cyan()));
    output.push_str(&format!("  {}  Match any char in set                     e.g. ls --re [Ff]ile  =>  File, file\n", "[abc]".bright_cyan()));
    output.push_str(&format!("  {}  Match any char NOT in set                 e.g. ls --re [^Ff]ile  =>  aile, bile...\n", "[^abc]".bright_cyan()));
    output.push_str(&format!("  {}  OR logic, match either expression         e.g. ls --re \\.rs$|\\.toml$  =>  .rs and .toml files\n", "|".bright_cyan()));
    output.push_str(&format!("  {}  Grouping for combining expressions         e.g. ls --re (src|target)\\/  =>  files under src or target\n\n", "()".bright_cyan()));
    
    output.push_str(&format!("  {}              Print current working directory\n", "pwd".cyan().bold()));
    output.push_str(&format!("  {}   Copy current directory path to clipboard\n", "cppwd".cyan().bold()));
    output.push_str(&format!("  {}  Copy file absolute path to clipboard\n\n", "cpf <file>".cyan().bold()));
    
    output.push_str(&format!("  {}            Change to home directory\n", "cd".cyan().bold()));
    output.push_str(&format!("  {}         Change to parent directory\n", "cd ..".cyan().bold()));
    output.push_str(&format!("  {}     Change to specified directory\n\n", "cd <path>".cyan().bold()));
    
    output.push_str(&format!("  {}         Open file with default application / Open directory in file explorer\n", "open <path>".cyan().bold()));
    
    output.push_str(&format!("  {}    Move file/folder to destination\n", "mv <source> <dest>".cyan().bold()));
    output.push_str(&format!("  {}    Copy file/folder to destination (preserves original)\n\n", "mv <source> <dest> --cp".cyan().bold()));
    
    output.push_str(&format!("  {}    Create a file (auto-creates parent directories)\n", "mkdf -f <path>".cyan().bold()));
    output.push_str(&format!("  {}      Create a directory\n", "mkdf -d <path>".cyan().bold()));
    output.push_str(&format!("  {}   Create a directory with parents\n", "mkdf -d -p <path>".cyan().bold()));
    output.push_str(&format!("  {}     Show mkdf command help\n\n", "mkdf -h/--help".cyan().bold()));
    output.push_str(&format!("  {}   Search recursively with regex\n", "ls --re-deep <pattern>".cyan().bold()));
    
    output.push_str(&format!("  {}             Exit the program\n", "exit".cyan().bold()));
    output.push_str(&format!("  {}            Clear the screen\n", "clear".cyan().bold()));
    output.push_str(&format!("  {}             Show this help\n", "help".cyan().bold()));
    output.push_str(&format!("  {}            Manage path aliases\n\n", "alias".cyan().bold()));

    output.push_str(&format!("{}\n", "⌨️ Keyboard Shortcuts:".bright_yellow().bold()));
    output.push_str(&format!("  {}        Clear current input line in REPL mode\n\n", "ESC".cyan().bold()));

    output.push_str(&format!("{}\n\n", "✨ Path Aliases:".bright_green().bold()));
    output.push_str(&format!("  Use {} prefix to use path aliases for faster navigation\n", "@".yellow().bold()));
    output.push_str("  Example:\n");
    output.push_str(&format!("    {}              Add alias for project directory\n", "alias add proj ~/projects".cyan()));
    output.push_str(&format!("    {}               List directory using alias\n", "ls @proj".cyan()));
    output.push_str(&format!("    {}        Navigate to subdirectory using alias\n", "cd @proj/rfe".cyan()));
    output.push_str(&format!("    {}                Open file using alias\n", "open @proj/rfe/src/main.rs".cyan()));
    output.push_str("  Aliases are saved persistently and available across sessions\n\n");

    output.push_str(&format!("{}\n\n", "✨ File Tags:".bright_green().bold()));
    output.push_str("  Add custom tags to files and directories for better organization\n");
    output.push_str("  Example:\n");
    output.push_str(&format!("    {}              Add tags 'work' and 'rust' to file\n", "tag add src/main.rs work rust".cyan()));
    output.push_str(&format!("    {}              Remove tag 'old' from file\n", "tag remove src/main.rs old".cyan()));
    output.push_str(&format!("    {}              Get all tags of file\n", "tag get src/main.rs".cyan()));
    output.push_str(&format!("    {}              List all files with tags\n", "tag list".cyan()));
    output.push_str(&format!("    {}              List files and their tags in current directory\n", "ls -tag".cyan()));
    output.push_str(&format!("    {}            List files in current directory tagged 'rust'\n", "ls -t rust".cyan()));
    output.push_str(&format!("    {}  Find files matching both 'rust' and 'doc' tags\n", "tag find rust doc".cyan()));
    output.push_str("  Supports regex matching, multi-tag queries, and automatic backup persistence\n\n");

    output.push_str(&format!("{}\n\n", "✨ Command Chain:".bright_green().bold()));
    output.push_str(&format!("  Use {} to chain commands with sequential execution and output passing\n", "->".yellow().bold()));
    output.push_str(&format!("  Example: {} pwd -> ls -> cd .. -> pwd\n", "$".bright_black()));
    output.push_str(&format!("  Use {} to continue execution even if previous command fails\n", "->!".yellow().bold()));
    output.push_str(&format!("  Example: {} cd non_exist! -> ls\n", "$".bright_black()));
    output.push_str(&format!("  Use {{}} as placeholder to insert previous command's output\n"));
    output.push_str(&format!("  Example: {} cppwd -> alias add desktop {{}}\n\n", "$".bright_black()));
    
    Ok((output, String::new()))
}