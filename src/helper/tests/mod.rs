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
use tempfile::TempDir;

/// 构造测试用 RfeHelper：管理器配置隔离到临时目录（不触碰真实用户配置），
/// 返回的 TempDir 守卫需由调用方持有至测试结束
fn create_helper() -> (TempDir, RfeHelper) {
    let config = TempDir::new().unwrap();
    let helper = RfeHelper {
        completer: FilenameCompleter::new(),
        alias_manager: Arc::new(Mutex::new(
            AliasManager::with_config_dir(config.path().to_path_buf()).unwrap(),
        )),
        tag_manager: Arc::new(Mutex::new(
            TagManager::with_config_dir(config.path().to_path_buf()).unwrap(),
        )),
        last_ls_items: Arc::new(Mutex::new(Vec::new())),
        completion_manager: CompletionManager::new(),
    };
    (config, helper)
}
