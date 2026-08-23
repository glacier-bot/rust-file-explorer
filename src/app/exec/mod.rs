//! 重型命令执行模块
//! 从 cli.rs 拆分出的 cpf/cd/ls/open/mv 命令的具体执行逻辑

use crate::app::pipeline::CommandResult;
use crate::managers::{alias::AliasManager, tag::TagManager};
use crate::models::FileInfo;
use std::sync::{Arc, Mutex};

mod cd;
mod ls;
mod misc;
mod open;

pub(crate) use self::cd::exec_cd;
pub(crate) use self::ls::exec_ls;
pub(crate) use self::misc::{exec_cpf, exec_mv};
pub(crate) use self::open::exec_open;

/// 单条命令的执行结果（与 cli::execute_single_command 的返回类型一致）
pub(crate) type ExecResult =
    Result<(CommandResult, String, String, Option<String>), Box<dyn std::error::Error>>;

/// 重型命令执行时共享的上下文（聚合各命令共用的引用）
pub(crate) struct ExecContext<'a> {
    pub(crate) input: &'a str,
    pub(crate) input_data: &'a str,
    pub(crate) parts: &'a [String],
    pub(crate) alias_manager: &'a Arc<Mutex<AliasManager>>,
    pub(crate) tag_manager: &'a Arc<Mutex<TagManager>>,
    pub(crate) last_ls_items: &'a Arc<Mutex<Vec<FileInfo>>>,
    pub(crate) previous_dir: Option<&'a str>,
}
