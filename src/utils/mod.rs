//! 工具函数模块
//! 包含格式化、终端处理和路径相关的工具函数

pub mod format;
pub mod terminal;
pub mod path;
pub mod moe;
pub mod version;

/// 智能分割命令行参数，支持引号包围的参数
/// 例如："cd 'my folder'" 会被分割为 ["cd", "my folder"]
pub fn split_command_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '"';
    
    for c in input.chars() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
            }
            '"' | '\'' if in_quote && c == quote_char => {
                in_quote = false;
            }
            ' ' if !in_quote => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    
    if !current.is_empty() {
        args.push(current);
    }
    
    args
}