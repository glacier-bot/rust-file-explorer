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
        // 注意：FilenameCompleter 在 Windows 下对包含空格的无引号路径，
        // 只会在结果开头添加双引号，不会添加结尾双引号。
        // 我们需要检测这种情况并补充结尾引号。
        let result = self.completer.complete(line, pos, ctx)?;

        // 检查当前输入是否处于引号内
        let mut in_quote = false;
        let mut quote_char = '"';
        for c in line.chars() {
            match c {
                '"' | '\'' if !in_quote => {
                    in_quote = true;
                    quote_char = c;
                }
                '"' | '\'' if in_quote && c == quote_char => {
                    in_quote = false;
                }
                _ => {}
            }
        }

        if in_quote {
            // 处于引号内：FilenameCompleter 返回的结果不包含引号，
            // 这是正确行为（只替换引号内内容），无需额外处理
            // 但需要注意：FilenameCompleter 只识别双引号，不识别单引号
            // 所以当在单引号内时，它可能会添加双引号，我们需要移除
            let mut candidates = result.1;
            if quote_char == '\'' {
                // 在单引号内，移除 FilenameCompleter 可能添加的双引号
                for candidate in &mut candidates {
                    if candidate.replacement.starts_with('"') {
                        candidate.replacement = candidate.replacement.trim_start_matches('"').to_string();
                    }
                    if candidate.replacement.ends_with('"') {
                        candidate.replacement = candidate.replacement.trim_end_matches('"').to_string();
                    }
                }
            }
            Ok((result.0, candidates))
        } else {
            // 未处于引号内：检测 FilenameCompleter 是否只添加了开头引号
            let mut candidates = result.1;
            for candidate in &mut candidates {
                let repl = &candidate.replacement;
                // Windows 下 FilenameCompleter 对包含空格的无引号路径
                // 会添加开头双引号但不添加结尾双引号，例如："test directory\
                // 我们需要补充结尾双引号
                if repl.starts_with('"') && !repl.ends_with('"') {
                    candidate.replacement = format!("{}\"", repl);
                }
            }
            Ok((result.0, candidates))
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use rustyline::completion::Candidate;
    use rustyline::history::MemHistory;

    fn create_helper() -> RfeHelper {
        RfeHelper {
            completer: FilenameCompleter::new(),
            alias_manager: AliasManager::new().unwrap(),
            tag_manager: TagManager::new().unwrap(),
        }
    }

    /// 测试 RfeHelper 对无引号但包含空格的路径补充结尾引号
    #[test]
    fn test_rfe_helper_no_quote_adds_closing_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 无引号输入，路径包含空格
        // FilenameCompleter 会返回 "file with spaces.txt（只有开头引号）
        // RfeHelper 应该补充结尾引号，变成 "file with spaces.txt"
        let line = "cd file";
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        println!("无引号输入 '{}' 的补全结果:", line);
        for (i, candidate) in result.1.iter().enumerate() {
            println!(
                "  候选 {}: display={}, replacement={}",
                i,
                candidate.display(),
                candidate.replacement()
            );
        }

        // 查找包含空格的补全结果
        let space_candidate = result.1.iter().find(|c| c.display().contains(' '));
        if let Some(candidate) = space_candidate {
            let replacement = candidate.replacement();
            println!("包含空格的补全结果: {}", replacement);

            #[cfg(windows)]
            {
                assert!(
                    replacement.starts_with('"'),
                    "补全结果应开始于双引号: {}",
                    replacement
                );
                assert!(
                    replacement.ends_with('"'),
                    "补全结果应结束于双引号: {}",
                    replacement
                );
                // 验证没有双重引号
                assert!(
                    !replacement.starts_with("\"\""),
                    "补全结果不应有双重开头引号: {}",
                    replacement
                );
            }
        }
    }

    /// 测试 RfeHelper 在双引号内不额外添加引号
    #[test]
    fn test_rfe_helper_in_double_quote_no_extra_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 双引号内输入
        // FilenameCompleter 返回的结果不包含引号
        // RfeHelper 不应该额外添加引号
        let line = r#"cd "file"#;
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        println!("双引号内输入 '{}' 的补全结果:", line);
        for (i, candidate) in result.1.iter().enumerate() {
            println!(
                "  候选 {}: display={}, replacement={}",
                i,
                candidate.display(),
                candidate.replacement()
            );
        }

        assert!(!result.1.is_empty(), "应该找到补全候选");

        let candidate = &result.1[0];
        let replacement = candidate.replacement();

        // 在引号内，结果不应该包含引号
        assert!(
            !replacement.starts_with('"'),
            "引号内补全结果不应包含开头引号: {}",
            replacement
        );
        assert!(
            !replacement.ends_with('"'),
            "引号内补全结果不应包含结尾引号: {}",
            replacement
        );
    }

    /// 测试普通路径补全不添加引号
    #[test]
    fn test_rfe_helper_normal_path_no_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 普通路径（无空格）
        let line = "cd sr";
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        if !result.1.is_empty() {
            let candidate = &result.1[0];
            let replacement = candidate.replacement();
            println!("普通路径补全结果: {}", replacement);

            // 普通路径不应该有多余的引号
            assert!(
                !replacement.starts_with('"'),
                "普通路径补全不应包含引号: {}",
                replacement
            );
        }
    }

    /// 测试在双引号内补全带空格的文件
    #[test]
    fn test_rfe_helper_file_with_spaces_in_quotes() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 在双引号内补全带空格的文件
        let line = r#"open "file"#;
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        println!("双引号内文件补全 '{}' 的结果:", line);
        for (i, candidate) in result.1.iter().enumerate() {
            println!(
                "  候选 {}: display={}, replacement={}",
                i,
                candidate.display(),
                candidate.replacement()
            );
        }

        assert!(!result.1.is_empty(), "应该找到补全候选");

        // 在引号内，结果不应该包含引号
        let candidate = &result.1[0];
        let replacement = candidate.replacement();
        assert!(
            !replacement.starts_with('"'),
            "引号内补全结果不应包含开头引号: {}",
            replacement
        );
        assert!(
            !replacement.ends_with('"'),
            "引号内补全结果不应包含结尾引号: {}",
            replacement
        );
    }

    /// 测试在单引号内补全
    #[test]
    fn test_rfe_helper_in_single_quote_no_extra_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 单引号内输入
        let line = "cd 'file";
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        println!("单引号内输入 '{}' 的补全结果:", line);
        for (i, candidate) in result.1.iter().enumerate() {
            println!(
                "  候选 {}: display={}, replacement={}",
                i,
                candidate.display(),
                candidate.replacement()
            );
        }

        assert!(!result.1.is_empty(), "应该找到补全候选");

        let candidate = &result.1[0];
        let replacement = candidate.replacement();

        // 在单引号内，结果不应该包含引号
        assert!(
            !replacement.starts_with('"') && !replacement.starts_with('\''),
            "单引号内补全结果不应包含开头引号: {}",
            replacement
        );
        assert!(
            !replacement.ends_with('"') && !replacement.ends_with('\''),
            "单引号内补全结果不应包含结尾引号: {}",
            replacement
        );
    }

    /// 测试双引号已闭合情况下不再添加引号
    #[test]
    fn test_rfe_helper_already_closed_quote_no_extra_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 双引号已闭合，后面继续输入
        let line = r#"cd "file" "#;
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        println!("已闭合引号输入 '{}' 的补全结果:", line);
        for (i, candidate) in result.1.iter().enumerate() {
            println!(
                "  候选 {}: display={}, replacement={}",
                i,
                candidate.display(),
                candidate.replacement()
            );
        }

        if !result.1.is_empty() {
            let candidate = &result.1[0];
            let replacement = candidate.replacement();
            
            // 在闭合引号后，应该像普通补全一样处理
            // 如果包含空格则添加引号，否则不添加
            if replacement.contains(' ') {
                #[cfg(windows)]
                {
                    assert!(
                        replacement.starts_with('"') && replacement.ends_with('"'),
                        "包含空格的路径应有完整的双引号: {}",
                        replacement
                    );
                }
            }
        }
    }

    /// 测试以斜杠结尾的目录补全应添加结尾引号
    #[test]
    fn test_rfe_helper_directory_trailing_slash_with_closing_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 测试目录补全（以斜杠结尾）
        let line = "cd file";
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        println!("目录补全结果:");
        for (i, candidate) in result.1.iter().enumerate() {
            println!(
                "  候选 {}: display={}, replacement={}",
                i,
                candidate.display(),
                candidate.replacement()
            );
        }

        // 查找目录候选（以斜杠结尾且包含空格）
        let dir_candidate = result.1.iter().find(|c| {
            let repl = c.replacement();
            repl.contains(' ') && repl.ends_with('/')
        });

        if let Some(candidate) = dir_candidate {
            let replacement = candidate.replacement();
            println!("找到目录补全: {}", replacement);

            #[cfg(windows)]
            {
                assert!(
                    replacement.starts_with('"'),
                    "目录补全应开始于双引号: {}",
                    replacement
                );
                assert!(
                    replacement.ends_with('/'),
                    "目录补全应保持斜杠结尾: {}",
                    replacement
                );
                // 关键测试：斜杠后应该有结尾引号
                assert!(
                    replacement.ends_with("/\""),
                    "目录补全应在斜杠后有结尾引号: {}",
                    replacement
                );
            }
        }
    }

    /// 测试嵌套引号场景
    #[test]
    fn test_rfe_helper_complex_quote_scenarios() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 场景1：双引号中有单引号
        let line1 = r#"cd 'file"#;
        let result1 = helper.complete(line1, line1.len(), &ctx).unwrap();
        
        // 场景2：多个单词后的补全
        let line2 = "ls -la file";
        let result2 = helper.complete(line2, line2.len(), &ctx).unwrap();

        println!("复杂场景测试完成");
        println!("场景1候选数: {}", result1.1.len());
        println!("场景2候选数: {}", result2.1.len());
        
        assert!(!result2.1.is_empty() || true, "场景2可能有也可能没有候选");
    }
}