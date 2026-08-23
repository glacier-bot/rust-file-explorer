//! 路径补全模块：cd -r 行号路径与 @alias 别名路径的层级补全

use crate::cache::get_cached_dir_entries;
use crate::managers::alias::AliasManager;
use crate::models::FileInfo;
use rustyline::completion::Pair;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::quoting::ensure_quoted;

/// cd -r <行号>/<子路径> 补全
pub fn complete_line_number_path(
    line: &str,
    pos: usize,
    last_ls_items: &Arc<Mutex<Vec<FileInfo>>>,
) -> Option<(usize, Vec<Pair>)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 || parts[0] != "cd" || parts[1] != "-r" {
        return None;
    }

    let path_part = parts[2];

    // 检查是否包含/或\分割行号和子路径
    if let Some(slash_pos) = path_part.find(|c: char| c == '/' || c == '\\') {
        let line_num_str = &path_part[..slash_pos];
        let original_sep = &path_part[slash_pos..=slash_pos]; // 保存用户使用的分隔符
        let sub_path = &path_part[slash_pos + 1..];

        // 解析行号
        if let Ok(line_num) = line_num_str.parse::<usize>() {
            let items = last_ls_items.lock().unwrap();
            if line_num >= 1 && line_num <= items.len() {
                let item = &items[line_num - 1];
                if item.is_dir {
                    let base_dir = Path::new(&item.full_path);

                    // 解析子路径，确定要浏览的目录（支持Unix和Windows路径分隔符）
                    let (dir_to_list, file_prefix) = if sub_path.ends_with('/') || sub_path.ends_with('\\')
                    {
                        (base_dir.join(sub_path), "")
                    } else if let Some(last_slash) = sub_path.rfind(|c: char| c == '/' || c == '\\') {
                        let dir_part = &sub_path[..last_slash];
                        let file_part = &sub_path[last_slash + 1..];
                        (base_dir.join(dir_part), file_part)
                    } else {
                        (base_dir.to_path_buf(), sub_path)
                    };

                    // 读取目录内容并提供补全
                    if dir_to_list.is_dir() {
                        let mut candidates = Vec::new();
                        if let Ok(dir_entries) = std::fs::read_dir(&dir_to_list) {
                            for entry in dir_entries.filter_map(|e| e.ok()) {
                                if let Some(name) = entry.file_name().to_str() {
                                    let is_dir = entry.metadata().ok().map(|m| m.is_dir()).unwrap_or(false);

                                    // 过滤匹配前缀的条目
                                    if !file_prefix.is_empty() && !name.starts_with(file_prefix) {
                                        continue;
                                    }

                                    // 构建补全路径，使用用户输入的分隔符
                                    let replacement = if let Some(last_slash) =
                                        sub_path.rfind(|c: char| c == '/' || c == '\\')
                                    {
                                        format!(
                                            "{}{}{}{}{}",
                                            line_num_str,
                                            original_sep,
                                            &sub_path[..last_slash],
                                            original_sep,
                                            name
                                        )
                                    } else {
                                        format!("{}{}{}", line_num_str, original_sep, name)
                                    };

                                    // 如果是目录，添加尾部斜杠（使用用户输入的分隔符）
                                    let replacement_with_sep = if is_dir {
                                        format!("{}{}", replacement, original_sep)
                                    } else {
                                        replacement.clone()
                                    };

                                    // 统一使用 ensure_quoted 处理引号
                                    candidates.push(Pair {
                                        display: name.to_string(),
                                        replacement: ensure_quoted(&replacement_with_sep),
                                    });
                                }
                            }
                        }

                        // 按目录在前、文件在后排序
                        candidates.sort_by(|a, b| {
                            let a_is_dir = a.replacement.trim_end_matches('"').ends_with('/');
                            let b_is_dir = b.replacement.trim_end_matches('"').ends_with('/');
                            match (a_is_dir, b_is_dir) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.display.cmp(&b.display),
                            }
                        });

                        if !candidates.is_empty() {
                            let start_pos = pos - path_part.len();
                            return Some((start_pos, candidates));
                        }
                    }
                }
            }
        }
    }

    None
}

/// 路径别名补全 - 支持 @alias/path 的层级补全
pub fn complete_alias_path(
    current_word: &str,
    _pos: usize,
    alias_manager: &Arc<Mutex<AliasManager>>,
) -> Option<(usize, Vec<Pair>)> {
    if let Some(at_pos) = current_word.rfind('@') {
        let after_at = &current_word[at_pos + 1..];

        // 检查是否包含路径分隔符（/或\），需要子路径补全
        if let Some(sep_pos) = after_at.find(|c: char| c == '/' || c == '\\') {
            let alias_name = &after_at[..sep_pos];
            let original_sep = &after_at[sep_pos..=sep_pos]; // 保存用户使用的分隔符
            let sub_path = &after_at[sep_pos + 1..];

            // 获取别名对应的真实路径
            let alias_manager = alias_manager.lock().unwrap();
            if let Some(alias_path) = alias_manager.get(alias_name) {
                let base_path = PathBuf::from(alias_path);

                // 解析子路径，确定要浏览的目录（支持两种分隔符）
                let (dir_to_list, file_prefix) = if sub_path.ends_with('/') || sub_path.ends_with('\\') {
                    (base_path.join(sub_path), "")
                } else if let Some(last_sep) = sub_path.rfind(|c: char| c == '/' || c == '\\') {
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
                                    let is_dir = entry.metadata().ok().map(|m| m.is_dir()).unwrap_or(false);
                                    new_entries.push((name.to_string(), is_dir));
                                }
                            }
                        }
                        crate::cache::cache_dir_entries(&dir_to_list, new_entries.clone());
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

                        // 构建补全路径，使用用户输入的分隔符
                        let replacement = if let Some(last_sep) = sub_path.rfind(|c: char| c == '/' || c == '\\') {
                            format!("@{}{}{}{}{}", alias_name, original_sep, &sub_path[..last_sep], original_sep, name)
                        } else {
                            format!("@{}{}{}", alias_name, original_sep, name)
                        };

                        // 如果是目录，添加尾部斜杠（使用用户输入的分隔符）
                        let replacement_with_sep = if is_dir {
                            format!("{}{}", replacement, original_sep)
                        } else {
                            replacement.clone()
                        };

                        // 统一使用 ensure_quoted 处理引号
                        candidates.push(Pair {
                            display: name.clone(),
                            replacement: ensure_quoted(&replacement_with_sep),
                        });
                    }

                    // 按目录在前、文件在后排序
                    candidates.sort_by(|a, b| {
                        let a_repl = a.replacement.trim_end_matches('"');
                        let a_is_dir = a_repl.ends_with('/') || a_repl.ends_with('\\');
                        let b_repl = b.replacement.trim_end_matches('"');
                        let b_is_dir = b_repl.ends_with('/') || b_repl.ends_with('\\');
                        match (a_is_dir, b_is_dir) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.display.cmp(&b.display),
                        }
                    });

                    if !candidates.is_empty() {
                        return Some((at_pos, candidates));
                    }
                }
            }
        } else {
            // 纯别名补全（无子路径）
            let alias_prefix = after_at;
            let mut candidates = Vec::new();
            let alias_manager = alias_manager.lock().unwrap();

            for (alias, path) in alias_manager.list() {
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
                for (alias, path) in alias_manager.list() {
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
                                            let is_dir = entry.metadata().ok().map(|m| m.is_dir()).unwrap_or(false);
                                            new_entries.push((name.to_string(), is_dir));
                                        }
                                    }
                                }
                                crate::cache::cache_dir_entries(&alias_path, new_entries.clone());
                                new_entries
                            };

                            let mut sub_candidates = Vec::new();
                            for (name, is_dir) in entries.into_iter().take(20) {
                                let replacement = if is_dir {
                                    format!("@{}/{}/", alias, name)
                                } else {
                                    format!("@{}/{}", alias, name)
                                };

                                // 统一使用 ensure_quoted 处理引号
                                sub_candidates.push(Pair {
                                    display: name,
                                    replacement: ensure_quoted(&replacement),
                                });
                            }
                            candidates.extend(sub_candidates);
                        }
                        break; // 只处理第一个匹配的别名
                    }
                }
            }

            if !candidates.is_empty() {
                return Some((at_pos, candidates));
            }
        }
    }
    None
}
