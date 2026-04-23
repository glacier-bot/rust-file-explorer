use arboard::Clipboard;
use colored::*;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::env;
use std::fs::{self, Metadata};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

struct FileInfo {
    name: String,
    icon: &'static str,
    color: Color,
    size: u64,
    created: Option<SystemTime>,
    modified: SystemTime,
    is_dir: bool,
}

struct RfeHelper {
    completer: FilenameCompleter,
}

impl Completer for RfeHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Helper for RfeHelper {}
impl Highlighter for RfeHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        if prompt.starts_with("rfe ") && prompt.ends_with(" >") {
            let dir = &prompt[4..prompt.len() - 2];
            let colored = format!(
                "{} {} {}",
                "rfe".bright_green().bold(),
                dir.bright_blue().bold(),
                ">".bright_blue().bold()
            );
            std::borrow::Cow::Owned(colored)
        } else {
            std::borrow::Cow::Borrowed(prompt)
        }
    }
}
impl Hinter for RfeHelper {
    type Hint = String;
}
impl Validator for RfeHelper {}

fn get_file_icon_and_color(path: &PathBuf, metadata: &Metadata) -> (&'static str, Color) {
    if metadata.is_dir() {
        ("📁", Color::BrightBlue)
    } else if metadata.is_symlink() {
        ("🔗", Color::Cyan)
    } else {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => ("🦀", Color::BrightRed),
            "toml" | "json" | "yaml" | "yml" => ("📋", Color::BrightYellow),
            "md" | "txt" => ("📝", Color::White),
            "gitignore" | "git" => ("🔀", Color::BrightMagenta),
            "exe" | "bin" => ("⚡", Color::BrightGreen),
            "jpg" | "jpeg" | "png" | "gif" | "svg" => ("📷", Color::Magenta),
            "mp3" | "wav" | "flac" => ("🎵", Color::BrightMagenta),
            "mp4" | "avi" | "mkv" => ("🎬", Color::Red),
            "zip" | "tar" | "gz" | "7z" | "rar" => ("📦", Color::BrightRed),
            "pdf" => ("📕", Color::Red),
            "doc" | "docx" => ("📘", Color::BrightBlue),
            "xls" | "xlsx" => ("📗", Color::BrightGreen),
            "ppt" | "pptx" => ("📙", Color::BrightYellow),
            "html" | "css" | "js" | "ts" => ("🌐", Color::BrightCyan),
            "py" => ("🐍", Color::BrightYellow),
            "go" => ("🐹", Color::BrightCyan),
            "java" => ("☕", Color::BrightRed),
            "c" | "cpp" | "h" | "hpp" => ("🔧", Color::BrightBlue),
            "sh" | "bat" | "ps1" => ("💻", Color::BrightGreen),
            "lock" => ("🔒", Color::BrightYellow),
            "log" => ("📜", Color::BrightBlack),
            _ => ("📄", Color::White),
        }
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:>5.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:>5.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:>5.1} KB", size as f64 / KB as f64)
    } else {
        format!("{:>6} B", size)
    }
}

fn format_time_absolute(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let total_secs = duration.as_secs();
            let mut days = total_secs / 86400;
            let secs_in_day = total_secs % 86400;
            let hours = secs_in_day / 3600;
            let mins = (secs_in_day % 3600) / 60;
            let secs = secs_in_day % 60;

            let mut year = 1970;
            while days >= if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                366
            } else {
                365
            } {
                days -= if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    366
                } else {
                    365
                };
                year += 1;
            }

            let mut month = 1;
            let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut mdays = month_days.iter();
            while let Some(&md) = mdays.next() {
                let adjust = if month == 2 && (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) {
                    1
                } else {
                    0
                };
                if days < md + adjust {
                    break;
                }
                days -= md + adjust;
                month += 1;
            }

            let day = days + 1;

            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hours, mins, secs
            )
        }
        Err(_) => "                   N/A".to_string(),
    }
}

fn truncate_string(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        return s.to_string();
    }

    let available_width = max_width.saturating_sub(3);
    if available_width == 0 {
        return "...".to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let c_width = c.width().unwrap_or(1);
        if current_width + c_width > available_width {
            break;
        }
        result.push(c);
        current_width += c_width;
    }

    result + "..."
}

fn is_hidden(path: &PathBuf) -> bool {
    #[cfg(unix)]
    {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        match fs::metadata(path) {
            Ok(meta) => (meta.file_attributes() & 2) != 0,
            Err(_) => false,
        }
    }
}

fn print_welcome() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".bright_green()
    );
    println!(
        "{}",
        "║           Rust File Explorer v0.1.0                          ║".bright_green()
    );
    println!(
        "{}",
        "║           A cross-platform CLI file browser                  ║".bright_green()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".bright_green()
    );
    println!();
    println!("{}", "Commands:".bright_yellow());
    println!("  {}  - List directory contents", "ls".cyan());
    println!("  {}  - Print working directory", "pwd".cyan());
    println!("  {}  - Copy current directory path to clipboard", "cppwd".cyan());
    println!("  {}  - Copy file absolute path to clipboard", "cpf <file>".cyan());
    println!("  {}  - Change directory", "cd <path>".cyan());
    println!("  {}  - Open file with default application", "open <file>".cyan());
    println!("  {}  - Move/copy file or folder (use --cp to copy)", "mv <source> <dest> [--cp]".cyan());
    println!("  {} - Exit the program", "exit".cyan());
    println!("  {} - Clear the screen", "clear".cyan());
    println!("  {}  - Show this help", "help".cyan());
    println!();
}

fn get_prompt_string() -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let dir_str = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");

    format!("rfe {} >", dir_str)
}

fn cmd_pwd() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    println!("{}", current_dir.display().to_string().bright_cyan());
    Ok(())
}

fn cmd_cppwd() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let path_str = cwd.to_string_lossy().to_string();

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("failed to access system clipboard: {}", e))?;
    clipboard
        .set_text(&path_str)
        .map_err(|e| format!("failed to copy to clipboard: {}", e))?;

    println!("{} {}", "✔ Copied to clipboard:".bright_green(), path_str.cyan());
    Ok(())
}

fn cmd_cpf(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(file_path);

    if !path.exists() {
        return Err(format!("file does not exist: {}", file_path).into());
    }

    if !path.is_file() {
        return Err(format!("not a file: {}", file_path).into());
    }

    let abs_path = fs::canonicalize(&path)
        .map_err(|e| format!("failed to resolve absolute path: {}", e))?;

    let path_str = if cfg!(windows) {
        let s = abs_path.to_string_lossy().to_string();
        if s.starts_with(r"\\?\") {
            s[4..].to_string()
        } else {
            s
        }
    } else {
        abs_path.to_string_lossy().to_string()
    };

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("failed to access system clipboard: {}", e))?;
    clipboard
        .set_text(&path_str)
        .map_err(|e| format!("failed to copy to clipboard: {}", e))?;

    println!("{} {}", "✔ Copied to clipboard:".bright_green(), path_str.cyan());
    Ok(())
}

fn cmd_cd(path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let target = match path {
        Some("..") => {
            let mut current = env::current_dir()?;
            current.pop();
            current
        }
        Some("~") | None => dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        Some(p) => PathBuf::from(p),
    };

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    if !target.is_dir() {
        return Err(format!("Not a directory: {}", target.display()).into());
    }

    env::set_current_dir(&target)?;
    println!("{} {}", "Changed to:".green(), target.display().to_string().cyan());
    Ok(())
}

fn cmd_ls(all: bool, long: bool, path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let target = match path {
        Some(p) => PathBuf::from(p),
        None => env::current_dir()?,
    };

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    println!(
        "{} {}",
        "📂 Directory:".bright_yellow().bold(),
        target.display().to_string().bright_cyan()
    );
    println!();

    let entries = fs::read_dir(target)?;
    let mut files: Vec<FileInfo> = Vec::new();
    let mut dirs: Vec<FileInfo> = Vec::new();

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if !all && is_hidden(&path) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        match entry.metadata() {
            Ok(meta) => {
                let (icon, color) = get_file_icon_and_color(&path, &meta);
                let created = meta.created().ok();
                let modified = meta.modified().unwrap_or_else(|_| SystemTime::now());

                let file_info = FileInfo {
                    name,
                    icon,
                    color,
                    size: meta.len(),
                    created,
                    modified,
                    is_dir: meta.is_dir(),
                };

                if meta.is_dir() {
                    dirs.push(file_info);
                } else {
                    files.push(file_info);
                }
            }
            Err(_) => {
                let file_info = FileInfo {
                    name,
                    icon: "❓",
                    color: Color::Red,
                    size: 0,
                    created: None,
                    modified: SystemTime::now(),
                    is_dir: false,
                };
                files.push(file_info);
            }
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let mut all_items = Vec::new();
    all_items.extend(dirs);
    all_items.extend(files);

    if long {
        let separator = "+-------------------------------------------+-----------------------+-----------------------+--------------+".bright_black();
        
        println!("{}", separator);
        println!(
            "{}",
            "|                  Name                     |      Created Date     |     Modified Date     |     Size     |"
                .bright_white()
                .bold()
        );
        println!("{}", separator);

        for item in &all_items {
            let created_str = item
                .created
                .map_or("                   N/A".to_string(), |t| format_time_absolute(t));
            let modified_str = format_time_absolute(item.modified);

            let display_name = truncate_string(&item.name, 37);

            let display_text = format!("{}  {}", item.icon, display_name);
            let display_width = display_text.width();
            let padding = if display_width < 41 {
                " ".repeat(41 - display_width)
            } else {
                String::new()
            };
            let padded_name = format!("{}{}", display_text, padding);

            println!(
                "| {} | {} | {} | {} |",
                padded_name.color(item.color).bold(),
                format!("{:21}", created_str).bright_cyan(),
                format!("{:21}", modified_str).bright_magenta(),
                format!("{:>12}", format_size(item.size)).bright_yellow().bold()
            );
        }

        println!("{}", separator);
    } else {
        for item in &all_items {
            let display_name = truncate_string(&item.name, 50);
            println!("  {} {}", item.icon, display_name.color(item.color).bold());
        }
    }

    println!();
    let total = all_items.len();
    let dir_count = all_items.iter().filter(|f| f.is_dir).count();
    let file_count = total - dir_count;

    println!(
        "{} {} directories, {} files",
        "📊".bright_green(),
        dir_count.to_string().bright_blue(),
        file_count.to_string().bright_cyan()
    );
    println!();

    Ok(())
}

fn cmd_open(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let target = PathBuf::from(path);

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    if target.is_dir() {
        return Err(format!("Cannot open a directory: {}", target.display()).into());
    }

    open::that(&target)?;
    println!(
        "{} {} {}",
        "✔ Opened".bright_green(),
        target.display().to_string().cyan(),
        "with default application".bright_green()
    );
    Ok(())
}

fn cmd_clear() -> Result<(), Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush()?;
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let entry_path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if entry_type.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&entry_path)?;
                fs::set_permissions(&dest_path, metadata.permissions())?;
            }
            
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                let metadata = fs::metadata(&entry_path)?;
                let mut perm = fs::metadata(&dest_path)?.permissions();
                perm.set_readonly(metadata.permissions().readonly());
                fs::set_permissions(&dest_path, perm)?;
            }
        }
    }

    Ok(())
}

fn cmd_mv(source: &str, destination: &str, copy: bool) -> Result<(), Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(source);
    let dest_path = PathBuf::from(destination);

    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source_path.display()).into());
    }

    let source_metadata = fs::metadata(&source_path)?;

    let final_dest = if dest_path.is_dir() {
        dest_path.join(source_path.file_name().ok_or("Invalid source path")?)
    } else {
        dest_path.clone()
    };

    if final_dest.exists() {
        return Err(format!("Destination path already exists: Please remove it first: {}", final_dest.display()).into());
    }

    if copy {
        if source_metadata.is_dir() {
            copy_dir_recursive(&source_path, &final_dest)?;
            println!(
                "{} Copied directory {} to {}",
                "✔".bright_green(),
                source_path.display().to_string().cyan(),
                final_dest.display().to_string().cyan()
            );
        } else {
            fs::copy(&source_path, &final_dest)?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perm = fs::metadata(&final_dest)?.permissions();
                perm.set_mode(source_metadata.permissions().mode());
                fs::set_permissions(&final_dest, perm)?;
            }
            
            #[cfg(windows)]
            {
                let mut perm = fs::metadata(&final_dest)?.permissions();
                perm.set_readonly(source_metadata.permissions().readonly());
                fs::set_permissions(&final_dest, perm)?;
            }

            println!(
                "{} Copied file {} to {}",
                "✔".bright_green(),
                source_path.display().to_string().cyan(),
                final_dest.display().to_string().cyan()
            );
        }
    } else {
        fs::rename(&source_path, &final_dest)?;
        println!(
            "{} Moved {} to {}",
            "✔".bright_green(),
            source_path.display().to_string().cyan(),
            final_dest.display().to_string().cyan()
        );
    }

    Ok(())
}

fn cmd_help() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "📖 Available Commands:".bright_yellow().bold());
    println!();
    println!("  {}               List contents of current directory", "ls".cyan().bold());
    println!("  {}            List with detailed information", "ls -l".cyan().bold());
    println!("  {}             List including hidden files", "ls -a".cyan().bold());
    println!("  {}       List contents of specified directory", "ls <path>".cyan().bold());
    println!();
    println!("  {}              Print current working directory", "pwd".cyan().bold());
    println!("  {}   Copy current directory path to clipboard", "cppwd".cyan().bold());
    println!("  {}  Copy file absolute path to clipboard", "cpf <file>".cyan().bold());
    println!();
    println!("  {}            Change to home directory", "cd".cyan().bold());
    println!("  {}         Change to parent directory", "cd ..".cyan().bold());
    println!("  {}     Change to specified directory", "cd <path>".cyan().bold());
    println!();
    println!("  {}         Open file with default application", "open <file>".cyan().bold());
    println!();
    println!("  {}    Move file/folder to destination", "mv <source> <dest>".cyan().bold());
    println!("  {}    Copy file/folder to destination (preserves original)", "mv <source> <dest> --cp".cyan().bold());
    println!();
    println!("  {}             Exit the program", "exit".cyan().bold());
    println!("  {}            Clear the screen", "clear".cyan().bold());
    println!("  {}             Show this help", "help".cyan().bold());
    println!();
    Ok(())
}

fn execute_command(input: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    if parts.is_empty() {
        return Ok(false);
    }

    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "pwd" => cmd_pwd()?,
        "cppwd" => cmd_cppwd()?,
        "cpf" => {
            let path = parts.get(1).copied().ok_or("Usage: cpf <file>")?;
            cmd_cpf(path)?;
        }
        "cd" => {
            let path = parts.get(1).copied();
            cmd_cd(path)?;
        }
        "ls" => {
            let mut all = false;
            let mut long = false;
            let mut path: Option<&str> = None;

            for &part in &parts[1..] {
                match part {
                    "-a" | "--all" => all = true,
                    "-l" | "--long" => long = true,
                    "-la" | "-al" => {
                        all = true;
                        long = true;
                    }
                    p => path = Some(p),
                }
            }
            cmd_ls(all, long, path)?;
        }
        "open" => {
            let path = parts.get(1).copied().ok_or("Usage: open <file>")?;
            cmd_open(path)?;
        }
        "mv" => {
            let mut source: Option<&str> = None;
            let mut destination: Option<&str> = None;
            let mut copy = false;

            for &part in &parts[1..] {
                if part == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(part);
                } else if destination.is_none() {
                    destination = Some(part);
                }
            }

            let source = source.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;
            let destination = destination.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;
            
            cmd_mv(source, destination, copy)?;
        }
        "exit" | "quit" | "q" => {
            println!("{}", "👋 Goodbye!".bright_green());
            return Ok(true);
        }
        "clear" | "cls" => cmd_clear()?,
        "help" | "?" => cmd_help()?,
        _ => {
            println!(
                "{} Command not found: {}. Type 'help' for available commands.",
                "❌".red(),
                cmd.cyan()
            );
        }
    }

    Ok(false)
}

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    print_welcome();

    let helper = RfeHelper {
        completer: FilenameCompleter::new(),
    };

    let mut rl = rustyline::Editor::new()?;
    rl.set_helper(Some(helper));

    loop {
        let prompt = get_prompt_string();
        match rl.readline(&prompt) {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                rl.add_history_entry(input);

                match execute_command(input) {
                    Ok(should_exit) => {
                        if should_exit {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "👋 Goodbye!".bright_green());
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "👋 Goodbye!".bright_green());
                break;
            }
            Err(e) => {
                eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        if let Err(e) = run_repl() {
            eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
            std::process::exit(1);
        }
        return Ok(());
    }

    let cmd = &args[1].to_lowercase();
    let result = match cmd.as_str() {
        "pwd" => cmd_pwd(),
        "cppwd" => cmd_cppwd(),
        "cpf" => {
            let path = args.get(2).map(|s| s.as_str()).ok_or("Usage: rfe cpf <file>")?;
            cmd_cpf(path)
        }
        "cd" => {
            let path = args.get(2).map(|s| s.as_str());
            cmd_cd(path)
        }
        "ls" => {
            let mut all = false;
            let mut long = false;
            let mut path: Option<&str> = None;

            for arg in &args[2..] {
                match arg.as_str() {
                    "-a" | "--all" => all = true,
                    "-l" | "--long" => long = true,
                    "-la" | "-al" => {
                        all = true;
                        long = true;
                    }
                    p => path = Some(p),
                }
            }
            cmd_ls(all, long, path)
        }
        "open" => {
            let path = args.get(2).map(|s| s.as_str()).ok_or("Usage: rfe open <file>")?;
            cmd_open(path)
        }
        "mv" => {
            let mut source: Option<&str> = None;
            let mut destination: Option<&str> = None;
            let mut copy = false;

            for arg in &args[2..] {
                if arg == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(arg);
                } else if destination.is_none() {
                    destination = Some(arg);
                }
            }

            let source = source.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            let destination = destination.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            
            cmd_mv(source, destination, copy)
        }
        "exit" => {
            println!("{}", "👋 Goodbye!".bright_green());
            Ok(())
        }
        "clear" => cmd_clear(),
        "help" => cmd_help(),
        _ => {
            println!(
                "{} Command not found: {}. Type 'rfe help' for available commands.",
                "❌".red(),
                cmd.cyan()
            );
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
        std::process::exit(1);
    }

    Ok(())
}
