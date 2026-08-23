//! 补全上下文类型

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
