//! 辅助功能模块的测试
//! 按测试覆盖的主题拆分为多个子模块

mod path_completion;
mod quote_edge_cases;
mod quote_policy;
mod quote_scenarios;

use super::RfeHelper;
use crate::completion::CompletionManager;
use crate::managers::alias::AliasManager;
use crate::managers::tag::TagManager;
use rustyline::completion::FilenameCompleter;
use std::sync::{Arc, Mutex};

fn create_helper() -> RfeHelper {
    RfeHelper {
        completer: FilenameCompleter::new(),
        alias_manager: Arc::new(Mutex::new(AliasManager::new().unwrap())),
        tag_manager: Arc::new(Mutex::new(TagManager::new().unwrap())),
        last_ls_items: Arc::new(Mutex::new(Vec::new())),
        completion_manager: CompletionManager::new(),
    }
}
