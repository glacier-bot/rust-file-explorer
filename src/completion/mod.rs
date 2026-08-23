//! 命令补全与提示模块
//! 负责管理所有命令的元数据、补全规则和输入提示

use std::collections::HashMap;

/// 命令参数类型
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ArgType {
    /// 标志参数（无值，如 -a, --all）
    Flag,
    /// 带值参数（如 -t <tag>）
    Value(String),
    /// 路径参数
    Path,
    /// 标签参数
    Tag,
    /// 别名参数
    Alias,
}

/// 命令参数定义
#[derive(Debug, Clone)]
pub struct CommandArg {
    /// 参数名（如 -a, --all）
    pub name: String,
    /// 参数别名（如 -a 的别名是 --all）
    pub aliases: Vec<String>,
    /// 参数类型
    pub arg_type: ArgType,
    /// 参数描述
    pub description: String,
}

impl CommandArg {
    pub fn new(name: &str, arg_type: ArgType, description: &str) -> Self {
        Self {
            name: name.to_string(),
            aliases: Vec::new(),
            arg_type,
            description: description.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn with_alias(mut self, alias: &str) -> Self {
        self.aliases.push(alias.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn matches(&self, input: &str) -> bool {
        self.name.starts_with(input) || self.aliases.iter().any(|a| a.starts_with(input))
    }

    pub fn all_names(&self) -> Vec<String> {
        let mut names = vec![self.name.clone()];
        names.extend(self.aliases.clone());
        names
    }
}

/// 命令定义
#[derive(Debug, Clone)]
pub struct CommandDef {
    /// 命令名
    pub name: String,
    /// 命令别名
    pub aliases: Vec<String>,
    /// 命令描述
    pub description: String,
    /// 命令参数列表
    pub args: Vec<CommandArg>,
    /// 是否接受路径参数
    pub accepts_path: bool,
    /// 子命令（如 alias add/remove/list）
    pub subcommands: Vec<CommandDef>,
}

impl CommandDef {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            aliases: Vec::new(),
            description: description.to_string(),
            args: Vec::new(),
            accepts_path: false,
            subcommands: Vec::new(),
        }
    }

    pub fn with_alias(mut self, alias: &str) -> Self {
        self.aliases.push(alias.to_string());
        self
    }

    pub fn with_arg(mut self, arg: CommandArg) -> Self {
        self.args.push(arg);
        self
    }

    pub fn with_path_support(mut self) -> Self {
        self.accepts_path = true;
        self
    }

    pub fn with_subcommand(mut self, subcmd: CommandDef) -> Self {
        self.subcommands.push(subcmd);
        self
    }

    pub fn matches_name(&self, input: &str) -> bool {
        self.name.starts_with(input) || self.aliases.iter().any(|a| a.starts_with(input))
    }

    pub fn all_names(&self) -> Vec<String> {
        let mut names = vec![self.name.clone()];
        names.extend(self.aliases.clone());
        names
    }

    /// 获取匹配的参数补全选项
    pub fn get_arg_completions(&self, prefix: &str) -> Vec<(String, String)> {
        let mut completions = Vec::new();
        for arg in &self.args {
            for name in arg.all_names() {
                if name.starts_with(prefix) {
                    completions.push((name, arg.description.clone()));
                }
            }
        }
        completions
    }

    /// 获取匹配的子命令补全选项
    pub fn get_subcommand_completions(&self, prefix: &str) -> Vec<(String, String)> {
        let mut completions = Vec::new();
        for subcmd in &self.subcommands {
            for name in subcmd.all_names() {
                if name.starts_with(prefix) {
                    completions.push((name, subcmd.description.clone()));
                }
            }
        }
        completions
    }
}

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

/// 命令补全管理器
#[derive(Debug, Clone)]
pub struct CompletionManager {
    commands: HashMap<String, CommandDef>,
    command_list: Vec<(String, String)>, // (name, description)
}

impl Default for CompletionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionManager {
    pub fn new() -> Self {
        let defs = build_command_definitions();
        let mut commands = HashMap::new();
        let mut command_list = Vec::new();

        for def in defs {
            command_list.push((def.name.clone(), def.description.clone()));
            for alias in &def.aliases {
                command_list.push((alias.clone(), def.description.clone()));
            }
            commands.insert(def.name.clone(), def);
        }

        Self {
            commands,
            command_list,
        }
    }

    /// 获取命令名补全
    pub fn get_command_completions(&self, prefix: &str) -> Vec<(String, String)> {
        self.command_list
            .iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .cloned()
            .collect()
    }

    /// 获取命令定义
    pub fn get_command(&self, name: &str) -> Option<&CommandDef> {
        // 先直接查找
        if let Some(cmd) = self.commands.get(name) {
            return Some(cmd);
        }
        // 再检查别名
        for cmd in self.commands.values() {
            if cmd.aliases.contains(&name.to_string()) {
                return Some(cmd);
            }
        }
        None
    }

    /// 解析输入行，获取当前应该补全什么
    pub fn parse_input_for_completion(&self, line: &str, pos: usize) -> CompletionContext {
        let line_up_to_pos = &line[..pos];
        let parts: Vec<&str> = line_up_to_pos.split_whitespace().collect();

        if parts.is_empty() {
            return CompletionContext::CommandName(String::new());
        }

        // 获取光标前的最后一个词
        let last_word = parts.last().unwrap_or(&"");
        let before_last_word = if parts.len() >= 2 {
            parts[parts.len() - 2]
        } else {
            ""
        };

        // 判断光标是否在一个词的中间
        let ends_with_space = line_up_to_pos.ends_with(' ');

        if parts.len() == 1 && !ends_with_space {
            // 正在输入命令名
            return CompletionContext::CommandName(parts[0].to_string());
        }

        // 有命令了，查找对应的命令定义
        let cmd_name = parts[0];
        if let Some(cmd) = self.get_command(cmd_name) {
            if parts.len() == 1 {
                // 刚输入完命令，可能需要参数补全
                return CompletionContext::CommandArg(cmd_name.to_string(), String::new());
            }

            // 检查是否有子命令
            if !cmd.subcommands.is_empty() {
                let subcmd_name = parts[1];
                if let Some(subcmd) = cmd.subcommands.iter().find(|s| s.matches_name(subcmd_name)) {
                    // 子命令匹配，检查是否需要子命令的参数补全
                    if parts.len() == 2 && !ends_with_space {
                        return CompletionContext::SubcommandArg(
                            cmd_name.to_string(),
                            subcmd_name.to_string(),
                            String::new(),
                        );
                    }
                    if parts.len() >= 2 && !ends_with_space {
                        let arg_prefix = parts.last().unwrap_or(&"");
                        if arg_prefix.starts_with('-') {
                            return CompletionContext::SubcommandArg(
                                cmd_name.to_string(),
                                subcmd_name.to_string(),
                                arg_prefix.to_string(),
                            );
                        }
                    }
                    // 子命令接受路径
                    if subcmd.accepts_path {
                        return CompletionContext::Path;
                    }
                } else if parts.len() == 2 && !ends_with_space {
                    // 正在输入子命令
                    return CompletionContext::Subcommand(cmd_name.to_string(), subcmd_name.to_string());
                }
            }

            // 检查是否正在输入参数（以 - 开头）
            if !ends_with_space && last_word.starts_with('-') {
                return CompletionContext::CommandArg(cmd_name.to_string(), last_word.to_string());
            }

            // 检查是否接受路径
            if cmd.accepts_path {
                return CompletionContext::Path;
            }

            // 检查前一个词是否是需要值的参数
            for arg in &cmd.args {
                if matches!(arg.arg_type, ArgType::Value(_))
                    && (arg.name == before_last_word || arg.aliases.contains(&before_last_word.to_string()))
                {
                    match &arg.arg_type {
                        ArgType::Value(hint) if hint.contains("tag") => return CompletionContext::Tag,
                        ArgType::Value(_) => return CompletionContext::Path,
                        _ => {}
                    }
                }
            }
        }

        CompletionContext::Unknown
    }
}

/// 补全文境
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContext {
    /// 正在输入命令名（带前缀）
    CommandName(String),
    /// 正在输入命令参数（命令名，参数前缀）
    CommandArg(String, String),
    /// 正在输入子命令（父命令名，子命令前缀）
    Subcommand(String, String),
    /// 正在输入子命令的参数
    SubcommandArg(String, String, String),
    /// 需要路径补全
    Path,
    /// 需要标签补全
    Tag,
    /// 未知类型
    Unknown,
}

/// 生成提示文本的样式
#[allow(dead_code)]
pub mod hint_style {
    use colored::*;

    /// 标准模式下的提示样式
    pub fn std_hint(text: &str) -> String {
        format!("{}", text.dimmed())
    }

    /// Moe 模式下的提示样式
    pub fn moe_hint(text: &str) -> String {
        format!("{}", text.truecolor(255, 182, 193).dimmed())
    }

    /// 根据模式选择提示样式
    pub fn moe_or_std(text: &str, is_moe: bool) -> String {
        if is_moe {
            moe_hint(text)
        } else {
            std_hint(text)
        }
    }

    /// 参数提示
    pub fn arg_hint(name: &str, desc: &str, is_moe: bool) -> String {
        if is_moe {
            format!(
                "  {}  {}",
                name.truecolor(255, 105, 180).bold(),
                desc.truecolor(255, 182, 193).dimmed()
            )
        } else {
            format!(
                "  {}  {}",
                name.bright_blue().bold(),
                desc.dimmed()
            )
        }
    }

    /// 命令提示
    pub fn cmd_hint(name: &str, desc: &str, is_moe: bool) -> String {
        if is_moe {
            format!(
                "  {}  {}",
                name.truecolor(255, 105, 180).bold(),
                desc.truecolor(255, 182, 193).dimmed()
            )
        } else {
            format!(
                "  {}  {}",
                name.bright_green().bold(),
                desc.dimmed()
            )
        }
    }
}
