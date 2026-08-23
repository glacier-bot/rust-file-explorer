//! 补全辅助函数模块
//! 提供路径补全相关的工具函数
//! 引号相关函数移至 quote_context，路径补全函数移至 path_completion，
//! 此处通过 re-export 保持原有引用路径不变

use rustyline::completion::Pair;
use std::sync::{Arc, Mutex};

pub use super::path_completion::{complete_alias_path, complete_line_number_path};
pub use super::quote_context::{
    apply_quote_policy, check_in_quote, is_after_closing_quote, is_before_closing_quote,
};

/// tag 命令专用的标签补全
pub fn complete_tag_command(
    line: &str,
    pos: usize,
    current_word: &str,
    tag_manager: &Arc<Mutex<crate::managers::tag::TagManager>>,
) -> Option<(usize, Vec<Pair>)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 && (parts[0] == "tag" || parts[0] == "t") {
        match parts[1] {
            "add" | "remove" | "rm" if parts.len() >= 3 => {
                let tag_prefix = current_word.split_whitespace().last().unwrap_or("");
                let mut candidates = Vec::new();
                let tag_manager = tag_manager.lock().unwrap();

                for tag in tag_manager.get_all_tags() {
                    if tag.starts_with(tag_prefix) {
                        candidates.push(Pair {
                            display: tag.clone(),
                            replacement: tag,
                        });
                    }
                }

                if !candidates.is_empty() {
                    let start_pos = pos - tag_prefix.len();
                    return Some((start_pos, candidates));
                }
            }
            _ => {}
        }
    }
    None
}
