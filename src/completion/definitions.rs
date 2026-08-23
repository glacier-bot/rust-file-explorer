//! 所有命令的补全定义数据表

use super::types::{ArgType, CommandArg, CommandDef};

/// 构建所有命令的定义
pub fn build_command_definitions() -> Vec<CommandDef> {
    vec![
        // ls 命令
        CommandDef::new("ls", "List directory contents")
            .with_alias("dir")
            .with_arg(CommandArg::new("-a", ArgType::Flag, "Show hidden files"))
            .with_arg(CommandArg::new("--all", ArgType::Flag, "Show hidden files"))
            .with_arg(CommandArg::new("-l", ArgType::Flag, "Show detailed information"))
            .with_arg(CommandArg::new("--long", ArgType::Flag, "Show detailed information"))
            .with_arg(CommandArg::new("-la", ArgType::Flag, "Show all files with details"))
            .with_arg(CommandArg::new("-al", ArgType::Flag, "Show all files with details"))
            .with_arg(CommandArg::new("--re", ArgType::Flag, "Search files by regex"))
            .with_arg(CommandArg::new("--re-deep", ArgType::Flag, "Recursive regex search"))
            .with_arg(CommandArg::new("--re-insensitive", ArgType::Flag, "Case-insensitive regex"))
            .with_arg(CommandArg::new("--xcaps", ArgType::Flag, "Case-insensitive regex alias"))
            .with_arg(CommandArg::new("-tag", ArgType::Flag, "Show file tags"))
            .with_arg(CommandArg::new("--tags", ArgType::Flag, "Show file tags"))
            .with_arg(CommandArg::new("-t", ArgType::Value("<tag_regex>".to_string()), "Filter by tag regex"))
            .with_arg(CommandArg::new("--tag", ArgType::Value("<tag_regex>".to_string()), "Filter by tag regex"))
            .with_path_support(),

        // cd 命令
        CommandDef::new("cd", "Change directory")
            .with_arg(CommandArg::new("-b", ArgType::Flag, "Go back to previous directory"))
            .with_arg(CommandArg::new("-back", ArgType::Flag, "Go back to previous directory"))
            .with_arg(CommandArg::new("-r", ArgType::Value("<line_number>".to_string()), "Cd by ls line number"))
            .with_arg(CommandArg::new("-tag", ArgType::Value("<tag>".to_string()), "Cd to directory with .index file matching tag"))
            .with_path_support(),

        // open 命令
        CommandDef::new("open", "Open file with default application")
            .with_arg(CommandArg::new("-r", ArgType::Value("<line_number>".to_string()), "Open by ls line number"))
            .with_arg(CommandArg::new("-tag", ArgType::Value("<tag>".to_string()), "Open directory matching tag"))
            .with_path_support(),

        // mv 命令
        CommandDef::new("mv", "Move or copy files")
            .with_arg(CommandArg::new("--cp", ArgType::Flag, "Copy instead of move"))
            .with_arg(CommandArg::new("-r", ArgType::Value("<line_number>".to_string()), "Use ls line number reference"))
            .with_path_support(),

        // pwd 命令
        CommandDef::new("pwd", "Print current working directory"),

        // cppwd 命令
        CommandDef::new("cppwd", "Copy current directory path to clipboard"),

        // cpf 命令
        CommandDef::new("cpf", "Copy file path to clipboard")
            .with_path_support(),

        // alias 命令
        CommandDef::new("alias", "Manage path aliases")
            .with_subcommand(
                CommandDef::new("add", "Add a new alias")
                    .with_alias("set")
            )
            .with_subcommand(
                CommandDef::new("remove", "Remove an alias")
                    .with_alias("rm")
                    .with_alias("delete")
            )
            .with_subcommand(
                CommandDef::new("list", "List all aliases")
                    .with_alias("ls")
            ),

        // tag 命令
        CommandDef::new("tag", "Manage file tags")
            .with_alias("t")
            .with_subcommand(
                CommandDef::new("add", "Add tags to a file")
                    .with_path_support()
            )
            .with_subcommand(
                CommandDef::new("remove", "Remove tags from a file")
                    .with_alias("rm")
                    .with_path_support()
            )
            .with_subcommand(
                CommandDef::new("clear", "Clear all tags from a file")
                    .with_path_support()
            )
            .with_subcommand(
                CommandDef::new("get", "Get tags of a file")
                    .with_path_support()
            )
            .with_subcommand(
                CommandDef::new("list", "List all tagged files")
                    .with_alias("ls")
            )
            .with_subcommand(
                CommandDef::new("find", "Find files by tag regex")
                    .with_alias("search")
            )
            .with_subcommand(
                CommandDef::new("backup", "Backup tag data")
            )
            .with_subcommand(
                CommandDef::new("restore", "Restore tag data from backup")
            ),

        // mkdf 命令
        CommandDef::new("mkdf", "Make directory or file")
            .with_arg(CommandArg::new("-d", ArgType::Flag, "Create directory"))
            .with_arg(CommandArg::new("-f", ArgType::Flag, "Create file"))
            .with_arg(CommandArg::new("-p", ArgType::Flag, "Create parent directories"))
            .with_path_support(),

        // change 命令
        CommandDef::new("change", "Change application settings")
            .with_arg(CommandArg::new("-std", ArgType::Flag, "Switch to standard mode"))
            .with_arg(CommandArg::new("--std", ArgType::Flag, "Switch to standard mode"))
            .with_arg(CommandArg::new("-moe", ArgType::Flag, "Switch to moe mode"))
            .with_arg(CommandArg::new("--moe", ArgType::Flag, "Switch to moe mode")),

        // clear 命令
        CommandDef::new("clear", "Clear terminal screen")
            .with_alias("cls"),

        // help 命令
        CommandDef::new("help", "Show help information")
            .with_alias("?")
            .with_alias("h"),

        // welcome 命令
        CommandDef::new("welcome", "Show welcome message"),

        // exit 命令
        CommandDef::new("exit", "Exit the application")
            .with_alias("quit")
            .with_alias("q"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_definition_uses_unified_tag_flag() {
        let defs = build_command_definitions();
        let cd = defs.iter().find(|d| d.name == "cd").unwrap();
        let flags: Vec<&str> = cd.args.iter().map(|a| a.name.as_str()).collect();
        assert!(flags.contains(&"-tag"), "cd should expose -tag: {:?}", flags);
        assert!(!flags.contains(&"-idx"), "cd must not expose -idx: {:?}", flags);
    }

    #[test]
    fn test_open_and_ls_tag_flags_unchanged() {
        let defs = build_command_definitions();
        let open = defs.iter().find(|d| d.name == "open").unwrap();
        assert!(open.args.iter().any(|a| a.name == "-tag"));
        let ls = defs.iter().find(|d| d.name == "ls").unwrap();
        assert!(ls.args.iter().any(|a| a.name == "-tag"));
        assert!(ls.args.iter().any(|a| a.name == "-t"));
    }
}
