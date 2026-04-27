//! 辅助功能模块
//! 包含命令补全和提示相关功能

use colored::Colorize;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::path::PathBuf;
use std::time::Instant;

use crate::cache::{cache_dir_entries, get_cached_dir_entries};
use crate::managers::alias::AliasManager;
use crate::managers::tag::TagManager;

/// RfeHelper 结构体
/// 实现了 rustyline 的各种辅助功能
pub struct RfeHelper {
    /// 文件名补全器
    pub completer: FilenameCompleter,
    /// 别名管理器
    pub alias_manager: AliasManager,
    /// 标签管理器
    pub tag_manager: TagManager,
}

impl Completer for RfeHelper {
    type Candidate = Pair;

    /// 实现命令补全功能
    /// 支持路径别名补全和标签补全
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let current_word = &line[..pos];
        
        // 路径别名补全 - 支持 @alias/path 的层级补全
        if let Some(at_pos) = current_word.rfind('@') {
            let after_at = &current_word[at_pos + 1..];
            
            // 检查是否包含路径分隔符（需要子路径补全）
            if let Some(sep_pos) = after_at.find('/') {
                let alias_name = &after_at[..sep_pos];
                let sub_path = &after_at[sep_pos + 1..];
                
                // 获取别名对应的真实路径
                if let Some(alias_path) = self.alias_manager.get(alias_name) {
                    let base_path = PathBuf::from(alias_path);
                    
                    // 解析子路径，确定要浏览的目录
                    let (dir_to_list, file_prefix) = if sub_path.ends_with('/') {
                        (base_path.join(sub_path), "")
                    } else if let Some(last_sep) = sub_path.rfind('/') {
                        let dir_part = &sub_path[..last_sep];
                        let file_part = &sub_path[last_sep + 1..];
                        (base_path.join(dir_part), file_part)
                    } else {
                        (base_path.clone(), sub_path)
                    };
                    
                    // 读取目录内容并提供补全（带缓存和性能限制）
                    if dir_to_list.is_dir() {
                        let start_time = Instant::now();
                        let mut candidates = Vec::new();
                        const MAX_COMPLETION_TIME_MS: u128 = 100;
                        const MAX_ENTRIES: usize = 100;
                        
                        // 尝试从缓存获取
                        let entries: Vec<(String, bool)> = if let Some(cached) = get_cached_dir_entries(&dir_to_list) {
                            cached
                        } else {
                            // 读取目录并缓存
                            let mut new_entries = Vec::new();
                            if let Ok(dir_entries) = std::fs::read_dir(&dir_to_list) {
                                for entry in dir_entries.filter_map(|e| e.ok()) {
                                    if let Some(name) = entry.file_name().to_str() {
                                        let is_dir = entry.metadata().ok()
                                            .map(|m| m.is_dir()).unwrap_or(false);
                                        new_entries.push((name.to_string(), is_dir));
                                    }
                                }
                            }
                            cache_dir_entries(&dir_to_list, new_entries.clone());
                            new_entries
                        };
                        
                        // 生成补全候选
                        for (name, is_dir) in entries {
                            // 性能检查：超时则返回已有结果
                            if start_time.elapsed().as_millis() > MAX_COMPLETION_TIME_MS {
                                break;
                            }
                            
                            // 过滤匹配前缀的条目
                            if !file_prefix.is_empty() && !name.starts_with(file_prefix) {
                                continue;
                            }
                            
                            // 限制最大条目数
                            if candidates.len() >= MAX_ENTRIES {
                                break;
                            }
                            
                            // 构建补全路径
                            let replacement = if let Some(last_sep) = sub_path.rfind('/') {
                                format!("@{}/{}/{}", alias_name, &sub_path[..last_sep], name)
                            } else {
                                format!("@{}/{}", alias_name, name)
                            };
                            
                            // 如果是目录，添加尾部斜杠
                            let replacement_with_sep = if is_dir {
                                format!("{}/", replacement)
                            } else {
                                replacement.clone()
                            };
                            
                            candidates.push(Pair {
                                display: name.clone(),
                                replacement: replacement_with_sep,
                            });
                        }
                        
                        // 按目录在前、文件在后排序
                        candidates.sort_by(|a, b| {
                            let a_is_dir = a.replacement.ends_with('/');
                            let b_is_dir = b.replacement.ends_with('/');
                            match (a_is_dir, b_is_dir) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.display.cmp(&b.display),
                            }
                        });
                        
                        if !candidates.is_empty() {
                            return Ok((at_pos, candidates));
                        }
                    }
                }
            } else {
                // 纯别名补全（无子路径）
                let alias_prefix = after_at;
                let mut candidates = Vec::new();
                
                for (alias, path) in self.alias_manager.list() {
                    if alias.starts_with(alias_prefix) {
                        candidates.push(Pair {
                            display: format!("📍 @{} -> {}", alias, path),
                            replacement: format!("@{}", alias),
                        });
                    }
                }
                
                // 如果有匹配的别名，同时提供别名的子路径补全
                if candidates.len() == 1 || alias_prefix.is_empty() {
                    // 获取第一个匹配别名的目录内容作为额外补全
                    for (alias, path) in self.alias_manager.list() {
                        if alias.starts_with(alias_prefix) {
                            let alias_path = PathBuf::from(path);
                            if alias_path.is_dir() {
                                // 使用缓存获取目录内容
                                let entries = if let Some(cached) = get_cached_dir_entries(&alias_path) {
                                    cached
                                } else {
                                    let mut new_entries = Vec::new();
                                    if let Ok(dir_entries) = std::fs::read_dir(&alias_path) {
                                        for entry in dir_entries.filter_map(|e| e.ok()) {
                                            if let Some(name) = entry.file_name().to_str() {
                                                let is_dir = entry.metadata().ok()
                                                    .map(|m| m.is_dir()).unwrap_or(false);
                                                new_entries.push((name.to_string(), is_dir));
                                            }
                                        }
                                    }
                                    cache_dir_entries(&alias_path, new_entries.clone());
                                    new_entries
                                };
                                
                                let mut sub_candidates = Vec::new();
                                for (name, is_dir) in entries.into_iter().take(20) {
                                    let replacement = if is_dir {
                                        format!("@{}/{}/", alias, name)
                                    } else {
                                        format!("@{}/{}", alias, name)
                                    };
                                    
                                    sub_candidates.push(Pair {
                                        display: name,
                                        replacement,
                                    });
                                }
                                candidates.extend(sub_candidates);
                            }
                            break; // 只处理第一个匹配的别名
                        }
                    }
                }
                
                if !candidates.is_empty() {
                    return Ok((at_pos, candidates));
                }
            }
        }
        
        // 标签补全：当命令是tag add/tag remove时补全标签名
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && (parts[0] == "tag" || parts[0] == "t") {
            match parts[1] {
                "add" | "remove" | "rm" if parts.len() >= 3 => {
                    // 当前正在输入标签
                    let tag_prefix = current_word.split_whitespace().last().unwrap_or("");
                    let mut candidates = Vec::new();
                    
                    for tag in self.tag_manager.get_all_tags() {
                        if tag.starts_with(tag_prefix) {
                            candidates.push(Pair {
                                display: tag.clone(),
                                replacement: tag,
                            });
                        }
                    }
                    
                    if !candidates.is_empty() {
                        let start_pos = pos - tag_prefix.len();
                        return Ok((start_pos, candidates));
                    }
                }
                _ => {}
            }
        }
        
        // 使用默认的文件名补全
        self.completer.complete(line, pos, ctx)
    }
}

impl Helper for RfeHelper {}

impl Highlighter for RfeHelper {
    /// 高亮提示信息
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        if prompt.starts_with("rfe ") && prompt.ends_with(" >") {
            let dir = &prompt[4..prompt.len() - 2];
            let colored = format!(
                "{} {} {}",
                "rfe".bright_green().bold(),
                dir.bright_blue().bold(),
                ">".bright_blue().bold()
            );
            std::borrow::Cow::Owned(colored)
        } else {
            std::borrow::Cow::Borrowed(prompt)
        }
    }
}

impl Hinter for RfeHelper {
    type Hint = String;
}

impl Validator for RfeHelper {}