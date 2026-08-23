//! 辅助功能模块
//! 包含命令补全和提示相关功能

use rustyline::completion::FilenameCompleter;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::sync::{Arc, Mutex};
use crate::completion::CompletionManager;
use crate::managers::alias::AliasManager;
use crate::managers::tag::TagManager;
use crate::models::FileInfo;

mod complete;
pub mod completion_helpers;
pub mod highlight;
pub mod hinter;
pub mod path_completion;
pub mod quote_context;
pub mod quoting;

use highlight::highlight_prompt;
use hinter::hint;

/// RfeHelper 结构体
/// 实现了 rustyline 的各种辅助功能
pub struct RfeHelper {
    /// 文件名补全器
    pub completer: FilenameCompleter,
    /// 别名管理器
    pub alias_manager: Arc<Mutex<AliasManager>>,
    /// 标签管理器
    pub tag_manager: Arc<Mutex<TagManager>>,
    /// 最近一次ls的条目
    pub last_ls_items: Arc<Mutex<Vec<FileInfo>>>,
    /// 命令补全管理器
    pub completion_manager: CompletionManager,
}

impl Helper for RfeHelper {}

impl Highlighter for RfeHelper {
    /// 高亮提示信息
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        std::borrow::Cow::Owned(highlight_prompt(prompt))
    }
}

impl Hinter for RfeHelper {
    type Hint = String;

    /// 提供输入提示（内联显示，可通过右方向键或 Tab 接受）
    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        hint(line, pos, &self.completion_manager)
    }
}

impl Validator for RfeHelper {}

#[cfg(test)]
mod tests;
