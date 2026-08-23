//! 补全类型定义：参数类型与命令定义结构

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
