//! 命令补全与提示模块
//! 负责管理所有命令的元数据、补全规则和输入提示

mod context;
mod definitions;
mod types;

#[allow(dead_code)]
pub mod hint_style;

use std::collections::HashMap;

pub use context::CompletionContext;
pub use definitions::build_command_definitions;
pub use types::{ArgType, CommandDef};

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
