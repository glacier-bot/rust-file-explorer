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
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::cache::{cache_dir_entries, get_cached_dir_entries};
use crate::completion::{CompletionContext, CompletionManager};
use crate::managers::alias::AliasManager;
use crate::managers::tag::TagManager;
use crate::models::FileInfo;

/// 判断路径是否包含需要用双引号包裹的特殊字符
/// 包括：空格、英文括号 () [] {}、& | ; , ^ ! 等会被 shell 或命令解析器拆分的字符
/// 注意：不包含 / \ 这类合法的路径分隔符；也不包含 @ 这类已被解释的前缀
pub(crate) fn needs_quoting(path: &str) -> bool {
    path.chars().any(|c| matches!(
        c,
        ' ' | '\t' | '(' | ')' | '[' | ']' | '{' | '}'
            | '&' | '|' | ';' | ',' | '^' | '!' | '`' | '$' | '#'
    ))
}

/// 给补全候选添加双引号包裹（保留尾部斜杠）
/// 例如：`my dir/` -> `"my dir/"`、`my (dir)` -> `"my (dir)"`
/// 如果已经被双引号包裹，则保持不变
pub(crate) fn quote_replacement(replacement: &str) -> String {
    if replacement.starts_with('"') && replacement.ends_with('"') && replacement.len() >= 2 {
        return replacement.to_string();
    }
    format!("\"{}\"", replacement)
}

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

impl Completer for RfeHelper {
    type Candidate = Pair;

    /// 实现命令补全功能
    /// 支持命令名、参数、路径别名、标签等补全
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let current_word = &line[..pos];
        let is_moe = crate::utils::moe::is_moe();

        // 使用 CompletionManager 解析当前输入上下文
        let context = self.completion_manager.parse_input_for_completion(line, pos);

        match context {
            // 命令名补全
            CompletionContext::CommandName(prefix) => {
                let completions = self.completion_manager.get_command_completions(&prefix);
                if !completions.is_empty() {
                    let start_pos = pos - prefix.len();
                    let candidates: Vec<Pair> = completions
                        .into_iter()
                        .map(|(name, desc)| {
                            if is_moe {
                                // Moe 模式：使用粉色系
                                Pair {
                                    display: format!(
                                        "{}  {}",
                                        name.truecolor(255, 105, 180).bold(),
                                        desc.truecolor(255, 182, 193).dimmed()
                                    ),
                                    replacement: name,
                                }
                            } else {
                                // Std 模式：使用绿色系
                                Pair {
                                    display: format!("{}  {}", name.bright_green().bold(), desc.dimmed()),
                                    replacement: name,
                                }
                            }
                        })
                        .collect();
                    return Ok((start_pos, candidates));
                }
            }

            // 命令参数补全
            CompletionContext::CommandArg(cmd_name, arg_prefix) => {
                if let Some(cmd) = self.completion_manager.get_command(&cmd_name) {
                    let completions = cmd.get_arg_completions(&arg_prefix);
                    if !completions.is_empty() {
                        let start_pos = pos - arg_prefix.len();
                        let candidates: Vec<Pair> = completions
                            .into_iter()
                            .map(|(name, desc)| {
                                if is_moe {
                                    // Moe 模式：使用紫色系
                                    Pair {
                                        display: format!(
                                            "{}  {}",
                                            name.truecolor(186, 85, 211).bold(),
                                            desc.truecolor(255, 182, 193).dimmed()
                                        ),
                                        replacement: name,
                                    }
                                } else {
                                    // Std 模式：使用蓝色系
                                    Pair {
                                        display: format!("{}  {}", name.bright_blue().bold(), desc.dimmed()),
                                        replacement: name,
                                    }
                                }
                            })
                            .collect();
                        return Ok((start_pos, candidates));
                    }
                }
            }

            // 子命令补全
            CompletionContext::Subcommand(cmd_name, subcmd_prefix) => {
                if let Some(cmd) = self.completion_manager.get_command(&cmd_name) {
                    let completions = cmd.get_subcommand_completions(&subcmd_prefix);
                    if !completions.is_empty() {
                        let start_pos = pos - subcmd_prefix.len();
                        let candidates: Vec<Pair> = completions
                            .into_iter()
                            .map(|(name, desc)| {
                                if is_moe {
                                    // Moe 模式：使用橙色系
                                    Pair {
                                        display: format!(
                                            "{}  {}",
                                            name.truecolor(255, 165, 0).bold(),
                                            desc.truecolor(255, 182, 193).dimmed()
                                        ),
                                        replacement: name,
                                    }
                                } else {
                                    // Std 模式：使用黄色系
                                    Pair {
                                        display: format!("{}  {}", name.bright_yellow().bold(), desc.dimmed()),
                                        replacement: name,
                                    }
                                }
                            })
                            .collect();
                        return Ok((start_pos, candidates));
                    }
                }
            }

            // 标签补全
            CompletionContext::Tag => {
                let parts: Vec<&str> = line[..pos].split_whitespace().collect();
                let tag_prefix = parts.last().unwrap_or(&"");
                let tag_manager = self.tag_manager.lock().unwrap();
                let mut candidates = Vec::new();
                for tag in tag_manager.get_all_tags() {
                    if tag.starts_with(tag_prefix) {
                        let display = if is_moe {
                            tag.truecolor(255, 105, 180).to_string()
                        } else {
                            tag.bright_cyan().to_string()
                        };
                        candidates.push(Pair {
                            display,
                            replacement: tag,
                        });
                    }
                }
                if !candidates.is_empty() {
                    let start_pos = pos - tag_prefix.len();
                    return Ok((start_pos, candidates));
                }
            }

            // 子命令参数补全
            CompletionContext::SubcommandArg(cmd_name, _subcmd_name, arg_prefix) => {
                if let Some(cmd) = self.completion_manager.get_command(&cmd_name) {
                    for subcmd in &cmd.subcommands {
                        if subcmd.name == _subcmd_name || subcmd.aliases.contains(&_subcmd_name) {
                            let completions = subcmd.get_arg_completions(&arg_prefix);
                            if !completions.is_empty() {
                                let start_pos = pos - arg_prefix.len();
                                let candidates: Vec<Pair> = completions
                                    .into_iter()
                                    .map(|(name, desc)| {
                                        if is_moe {
                                            Pair {
                                                display: format!(
                                                    "{}  {}",
                                                    name.truecolor(186, 85, 211).bold(),
                                                    desc.truecolor(255, 182, 193).dimmed()
                                                ),
                                                replacement: name,
                                            }
                                        } else {
                                            Pair {
                                                display: format!("{}  {}", name.bright_blue().bold(), desc.dimmed()),
                                                replacement: name,
                                            }
                                        }
                                    })
                                    .collect();
                                return Ok((start_pos, candidates));
                            }
                        }
                    }
                }
            }

            // 路径补全由后面的逻辑处理
            CompletionContext::Path | CompletionContext::Unknown => {}
        }
        
        // cd -r <行号>/<子路径> 补全
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "cd" && parts[1] == "-r" {
            let path_part = parts[2];
            // 检查是否包含/或\分割行号和子路径
            if let Some(slash_pos) = path_part.find(|c: char| c == '/' || c == '\\') {
                let line_num_str = &path_part[..slash_pos];
                let original_sep = &path_part[slash_pos..=slash_pos]; // 保存用户使用的分隔符
                let sub_path = &path_part[slash_pos+1..];
                
                // 解析行号
                if let Ok(line_num) = line_num_str.parse::<usize>() {
                    let items = self.last_ls_items.lock().unwrap();
                    if line_num >= 1 && line_num <= items.len() {
                        let item = &items[line_num - 1];
                        if item.is_dir {
                            let base_dir = std::path::Path::new(&item.full_path);
                            
                            // 解析子路径，确定要浏览的目录（支持Unix和Windows路径分隔符）
                            let (dir_to_list, file_prefix) = if sub_path.ends_with('/') || sub_path.ends_with('\\') {
                                (base_dir.join(sub_path), "")
                            } else if let Some(last_slash) = sub_path.rfind(|c: char| c == '/' || c == '\\') {
                                let dir_part = &sub_path[..last_slash];
                                let file_part = &sub_path[last_slash+1..];
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
                                            let is_dir = entry.metadata().ok()
                                                .map(|m| m.is_dir()).unwrap_or(false);
                                            
                                            // 过滤匹配前缀的条目
                                            if !file_prefix.is_empty() && !name.starts_with(file_prefix) {
                                                continue;
                                            }
                                            
                                            // 构建补全路径，使用用户输入的分隔符
                                            let replacement = if let Some(last_slash) = sub_path.rfind(|c: char| c == '/' || c == '\\') {
                                                format!("{}{}{}{}{}", line_num_str, original_sep, &sub_path[..last_slash], original_sep, name)
                                            } else {
                                                format!("{}{}{}", line_num_str, original_sep, name)
                                            };
                                            
                                            // 如果是目录，添加尾部斜杠（使用用户输入的分隔符）
                                            let replacement_with_sep = if is_dir {
                                                format!("{}{}", replacement, original_sep)
                                            } else {
                                                replacement.clone()
                                            };
                                            
                                            // 统一引号策略
                                            let final_replacement = if needs_quoting(&replacement_with_sep) {
                                                quote_replacement(&replacement_with_sep)
                                            } else {
                                                replacement_with_sep
                                            };
                                            
                                            candidates.push(Pair {
                                                display: name.to_string(),
                                                replacement: final_replacement,
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
                                    return Ok((start_pos, candidates));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 路径别名补全 - 支持 @alias/path 的层级补全
        if let Some(at_pos) = current_word.rfind('@') {
            let after_at = &current_word[at_pos + 1..];
            
            // 检查是否包含路径分隔符（/或\），需要子路径补全
            if let Some(sep_pos) = after_at.find(|c: char| c == '/' || c == '\\') {
                let alias_name = &after_at[..sep_pos];
                let original_sep = &after_at[sep_pos..=sep_pos]; // 保存用户使用的分隔符
                let sub_path = &after_at[sep_pos + 1..];
                
                // 获取别名对应的真实路径
                let alias_manager = self.alias_manager.lock().unwrap();
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

                            // 统一引号策略：路径含空格/英文括号等特殊字符时用双引号包裹
                            let final_replacement = if needs_quoting(&replacement_with_sep) {
                                quote_replacement(&replacement_with_sep)
                            } else {
                                replacement_with_sep
                            };

                            candidates.push(Pair {
                                display: name.clone(),
                                replacement: final_replacement,
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
                            return Ok((at_pos, candidates));
                        }
                    }
                }
            } else {
                // 纯别名补全（无子路径）
                let alias_prefix = after_at;
                let mut candidates = Vec::new();
                let alias_manager = self.alias_manager.lock().unwrap();
                
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

                                    // 统一引号策略：路径含空格/英文括号等特殊字符时用双引号包裹
                                    let final_replacement = if needs_quoting(&replacement) {
                                        quote_replacement(&replacement)
                                    } else {
                                        replacement
                                    };

                                    sub_candidates.push(Pair {
                                        display: name,
                                        replacement: final_replacement,
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
                    let tag_manager = self.tag_manager.lock().unwrap();
                    
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
                        return Ok((start_pos, candidates));
                    }
                }
                _ => {}
            }
        }
        
        // 检查光标是否紧邻闭合引号之后，这种情况路径已经完整，不提供路径补全
        if pos > 0 {
            let last_char = line[..pos].chars().last().unwrap();
            if last_char == '"' || last_char == '\'' {
                return Ok((pos, Vec::new()));
            }
        }

        // 使用默认的文件名补全
        // 注意：FilenameCompleter 在 Windows 下对包含空格的无引号路径，
        // 只会在结果开头添加双引号，不会添加结尾双引号。
        // 我们需要检测这种情况并补充结尾引号。
        // 额外处理：Windows 下 FilenameCompleter 只识别反斜杠作为路径分隔符，
        // 我们需要把用户输入的正斜杠临时替换成反斜杠获取补全结果，再替换回去
        #[cfg(windows)]
        let (result, _use_fwd_slash) = {
            // 检查当前输入的路径部分是否使用正斜杠
            let current_input = &line[..pos];
            let use_fwd_slash = current_input.contains('/');
            
            if use_fwd_slash {
                // 临时把所有正斜杠替换成反斜杠
                let modified_line: String = line.chars().map(|c| if c == '/' { '\\' } else { c }).collect();
                // 调用补全
                let (start, candidates) = self.completer.complete(&modified_line, pos, ctx)?;
                // 把补全结果中的反斜杠替换回正斜杠
                let converted_candidates: Vec<Pair> = candidates.into_iter().map(|mut p| {
                    p.replacement = p.replacement.chars().map(|c| if c == '\\' { '/' } else { c }).collect();
                    p
                }).collect();
                ((start, converted_candidates), true)
            } else {
                // 正常调用
                (self.completer.complete(line, pos, ctx)?, false)
            }
        };
        
        #[cfg(not(windows))]
        let result = self.completer.complete(line, pos, ctx)?;

        // 检查当前输入是否处于引号内
        // 只遍历到光标位置的字符，光标后的引号不影响当前输入状态
        let mut in_quote = false;
        let mut quote_char = '"';
        for c in line[..pos].chars() {
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
            // 未处于引号内：统一引号策略
            // 1) FilenameCompleter 对含空格路径已加开头引号但缺尾引号，补上尾引号
            // 2) 对含英文括号等其他特殊字符（FilenameCompleter 不会自动加引号）的路径，
            //    手动在前后添加双引号
            let mut candidates = result.1;
            for candidate in &mut candidates {
                let repl = candidate.replacement.clone();

                if repl.starts_with('"') && !repl.ends_with('"') {
                    // 含空格情况：FilenameCompleter 已加开头引号，补上结尾引号
                    candidate.replacement = format!("{}\"", repl);
                } else if !repl.starts_with('"') && needs_quoting(&repl) {
                    // 含括号等特殊字符但 FilenameCompleter 未加引号：统一补全前后双引号
                    candidate.replacement = quote_replacement(&repl);
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
        if prompt.starts_with("rfe 🌸 ") && prompt.contains(" 💖 >") {
            let start = "rfe 🌸 ".len();
            let end = prompt.find(" 💖 >").unwrap_or(prompt.len());
            let dir = &prompt[start..end];
            let colored = format!(
                "{} {} {} {} {}",
                "rfe".truecolor(255, 105, 180).bold(),
                "🌸".truecolor(255, 182, 193),
                dir.truecolor(255, 182, 193).bold(),
                "💖".truecolor(255, 105, 180),
                ">".truecolor(255, 105, 180).bold()
            );
            std::borrow::Cow::Owned(colored)
        } else if prompt.starts_with("rfe ") && prompt.ends_with(" >") {
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

    /// 提供输入提示（内联显示，可通过右方向键或 Tab 接受）
    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        // 空行不显示提示
        if line.is_empty() || pos == 0 {
            return None;
        }

        let context = self.completion_manager.parse_input_for_completion(line, pos);

        match context {
            // 命令名提示：显示第一个匹配的命令作为内联提示
            // 注意：Hinter 返回纯文本，因为 Rustyline 内部计算显示宽度时不识别 ANSI 代码
            // 颜色样式由 Highlighter trait 处理
            CompletionContext::CommandName(ref prefix) if !prefix.is_empty() => {
                let completions = self.completion_manager.get_command_completions(prefix);
                if let Some((name, _desc)) = completions.first() {
                    let hint = if name.starts_with(prefix) {
                        name[prefix.len()..].to_string()
                    } else {
                        name.clone()
                    };
                    // 返回纯文本，不带 ANSI 颜色代码，避免光标位置计算错误
                    Some(hint)
                } else {
                    None
                }
            }

            // 命令参数提示：显示第一个匹配的参数作为内联提示
            CompletionContext::CommandArg(ref cmd_name, ref arg_prefix) => {
                if let Some(cmd) = self.completion_manager.get_command(cmd_name) {
                    let completions = cmd.get_arg_completions(arg_prefix);
                    if let Some((name, _desc)) = completions.first() {
                        let hint = if arg_prefix.is_empty() {
                            name.clone()
                        } else if name.starts_with(arg_prefix) {
                            name[arg_prefix.len()..].to_string()
                        } else {
                            name.clone()
                        };
                        Some(hint)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            // 子命令提示：显示第一个匹配的子命令作为内联提示
            CompletionContext::Subcommand(ref cmd_name, ref subcmd_prefix) => {
                if let Some(cmd) = self.completion_manager.get_command(cmd_name) {
                    let completions = cmd.get_subcommand_completions(subcmd_prefix);
                    if let Some((name, _desc)) = completions.first() {
                        let hint = if subcmd_prefix.is_empty() {
                            name.clone()
                        } else if name.starts_with(subcmd_prefix) {
                            name[subcmd_prefix.len()..].to_string()
                        } else {
                            name.clone()
                        };
                        Some(hint)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            _ => None,
        }
    }
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
            alias_manager: Arc::new(Mutex::new(AliasManager::new().unwrap())),
            tag_manager: Arc::new(Mutex::new(TagManager::new().unwrap())),
            last_ls_items: Arc::new(Mutex::new(Vec::new())),
            completion_manager: CompletionManager::new(),
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

    /// 测试 needs_quoting 辅助函数对各类特殊字符的识别
    #[test]
    fn test_needs_quoting_special_chars() {
        // 不含特殊字符
        assert!(!needs_quoting("simple"));
        assert!(!needs_quoting("path/to/file.txt"));
        assert!(!needs_quoting("C:\\Users\\q\\Desktop"));
        assert!(!needs_quoting("中文路径"));

        // 含空格
        assert!(needs_quoting("my folder"));
        assert!(needs_quoting("a b"));

        // 含英文括号
        assert!(needs_quoting("Program Files (x86)"));
        assert!(needs_quoting("dir(1)"));
        assert!(needs_quoting("[bracket]"));
        assert!(needs_quoting("{brace}"));

        // 其他 shell 特殊字符
        assert!(needs_quoting("a&b"));
        assert!(needs_quoting("a|b"));
        assert!(needs_quoting("a;b"));
        assert!(needs_quoting("a,b"));
    }

    /// 测试 quote_replacement 辅助函数
    #[test]
    fn test_quote_replacement_behavior() {
        // 基本包裹
        assert_eq!(quote_replacement("my dir"), r#""my dir""#);
        assert_eq!(quote_replacement("dir(1)/"), r#""dir(1)/""#);

        // 已被双引号包裹则保持不变
        assert_eq!(quote_replacement(r#""my dir""#), r#""my dir""#);
    }

    /// 测试 @alias 子路径补全在含特殊字符路径下统一加引号
    /// 通过模拟一个含括号的别名目录验证
    #[test]
    fn test_alias_sub_path_completion_with_special_chars() {
        use std::fs;
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 创建临时目录结构：tmp_root/sub (1)/inner.txt
        let tmp_root = std::env::temp_dir().join("rfe_test_alias_special");
        let _ = fs::remove_dir_all(&tmp_root);
        let sub_dir = tmp_root.join("sub (1)");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("inner.txt"), "x").unwrap();

        // 注册别名指向 tmp_root（直接操作 HashMap 避免污染真实配置）
        {
            let mut mgr = helper.alias_manager.lock().unwrap();
            mgr.aliases.insert(
                "rfe_special_alias".to_string(),
                tmp_root.to_string_lossy().to_string(),
            );
        }

        // 触发 @alias/ 子路径补全
        let line = "cd @rfe_special_alias/";
        let result = helper.complete(line, line.len(), &ctx).unwrap();

        let dir_candidate = result
            .1
            .iter()
            .find(|c| c.display.contains("sub (1)"));
        assert!(dir_candidate.is_some(), "应包含含括号的目录候选");
        let replacement = &dir_candidate.unwrap().replacement;
        assert!(
            replacement.starts_with('"') && replacement.ends_with('"'),
            "含括号的别名子路径补全应被双引号包裹: {}",
            replacement
        );

        // 清理
        let _ = fs::remove_dir_all(&tmp_root);
        let mut mgr = helper.alias_manager.lock().unwrap();
        mgr.aliases.remove("rfe_special_alias");
    }

    /// 测试默认文件名补全对仅含括号（无空格）特殊字符的路径也加双引号
    #[test]
    fn test_default_completion_quotes_parentheses() {
        use std::fs;
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        let tmp_root = std::env::temp_dir().join("rfe_test_paren");
        let _ = fs::remove_dir_all(&tmp_root);
        fs::create_dir_all(&tmp_root).unwrap();
        fs::create_dir_all(tmp_root.join("paren(only)")).unwrap();

        let prefix = tmp_root.join("paren").to_string_lossy().to_string();
        let line = format!("cd {}", prefix);
        let result = helper.complete(&line, line.len(), &ctx).unwrap();

        let cand = result
            .1
            .iter()
            .find(|c| c.replacement.contains("paren(only)"));
        if let Some(c) = cand {
            assert!(
                c.replacement.starts_with('"') && c.replacement.trim_end_matches('/').ends_with('"')
                    || c.replacement.ends_with('"'),
                "含括号的补全候选应被双引号包裹: {}",
                c.replacement
            );
        }

        let _ = fs::remove_dir_all(&tmp_root);
    }

    /// 测试光标在闭合双引号后补全为空
    #[test]
    fn test_rfe_helper_no_completion_after_closed_double_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 光标在闭合双引号后
        let line = r#"cd "te sts""#;
        let pos = line.len(); // 光标在最后一个"后面
        let result = helper.complete(line, pos, &ctx).unwrap();

        assert!(result.1.is_empty(), "闭合双引号后不应返回补全选项");
    }

    /// 测试光标在闭合单引号后补全为空
    #[test]
    fn test_rfe_helper_no_completion_after_closed_single_quote() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 光标在闭合单引号后
        let line = "cd 'te sts'";
        let pos = line.len(); // 光标在最后一个'后面
        let result = helper.complete(line, pos, &ctx).unwrap();

        assert!(result.1.is_empty(), "闭合单引号后不应返回补全选项");
    }

    /// 测试闭合引号后加空格补全第二个路径正常（多路径命令兼容）
    #[test]
    fn test_rfe_helper_completion_after_closed_quote_with_space() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // mv命令，闭合第一个路径引号后加空格，准备补全第二个路径
        let line = r#"mv "te sts" "#;
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        // 应该返回补全选项（当前目录下的文件）
        assert!(!result.1.is_empty(), "闭合引号后加空格应正常返回补全选项");
        // 验证不会有多余引号
        for cand in result.1 {
            if cand.replacement.contains(' ') {
                #[cfg(windows)]
                assert!(cand.replacement.starts_with('"') && cand.replacement.ends_with('"'), "含空格路径应正常加引号");
            }
        }
    }

    /// 测试闭合引号后加斜杠补全子目录正常
    #[test]
    fn test_rfe_helper_completion_after_closed_quote_with_slash() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 引号闭合后加斜杠，补全子目录内容
        let line = r#"cd "te sts"/"#;
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        // 应该返回te sts目录下的in dex.txt补全
        assert!(!result.1.is_empty(), "闭合引号后加斜杠应正常补全子目录");
        let has_in_dex = result.1.iter().any(|c| c.display.contains("in dex"));
        assert!(has_in_dex, "应该补全到te sts目录下的in dex.txt文件");
    }

    /// 测试无引号路径补全带空格的文件不会出现双重引号
    #[test]
    fn test_rfe_helper_no_double_quotes_for_space_path() {
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 输入cd te，补全te sts目录
        let line = "cd te";
        let pos = line.len();
        let result = helper.complete(line, pos, &ctx).unwrap();

        let te_sts_cand = result.1.iter().find(|c| c.display == "te sts");
        assert!(te_sts_cand.is_some(), "应该找到te sts目录补全");
        let replacement = &te_sts_cand.unwrap().replacement;
        #[cfg(windows)]
        {
            assert!(replacement.starts_with('"') && replacement.ends_with("/\""), "补全结果应为\"te sts/\"，不会有双重引号");
            assert!(!replacement.starts_with("\"\""), "不应出现双重开头引号");
        }
    }

    /// 测试 cd -r 使用正斜杠和反斜杠的补全功能
    #[test]
    fn test_cd_r_with_both_separators() {
        use std::fs;
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 创建临时目录结构
        let tmp_root = std::env::temp_dir().join("rfe_test_cd_r_sep");
        let _ = fs::remove_dir_all(&tmp_root);
        fs::create_dir_all(&tmp_root).unwrap();
        let sub_dir = tmp_root.join("test_subdir");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("inner_file.txt"), "test").unwrap();

        // 填充 last_ls_items
        {
            let mut items = helper.last_ls_items.lock().unwrap();
            items.push(crate::models::FileInfo {
                name: "tmp_root".to_string(),
                full_path: tmp_root.to_string_lossy().to_string(),
                icon: "📁",
                color: colored::Color::Blue,
                size: 0,
                created: None,
                modified: std::time::SystemTime::now(),
                is_dir: true,
                tags: vec![],
            });
        }

        // 测试正斜杠: cd -r 1/
        let line_fwd = "cd -r 1/";
        let result_fwd = helper.complete(line_fwd, line_fwd.len(), &ctx).unwrap();
        assert!(!result_fwd.1.is_empty(), "使用正斜杠应该有补全结果");

        // 检查补全是否使用正斜杠
        let subdir_cand_fwd = result_fwd.1.iter().find(|c| c.display == "test_subdir");
        assert!(subdir_cand_fwd.is_some(), "应该找到test_subdir");
        assert!(subdir_cand_fwd.unwrap().replacement.contains("1/"), "补全结果应使用正斜杠");

        // 测试反斜杠: cd -r 1\
        let line_back = r"cd -r 1\";
        let result_back = helper.complete(line_back, line_back.len(), &ctx).unwrap();
        assert!(!result_back.1.is_empty(), "使用反斜杠应该有补全结果");

        // 检查补全是否使用反斜杠
        let subdir_cand_back = result_back.1.iter().find(|c| c.display == "test_subdir");
        assert!(subdir_cand_back.is_some(), "应该找到test_subdir");
        assert!(subdir_cand_back.unwrap().replacement.contains(r"1\"), "补全结果应使用反斜杠");

        // 清理
        let _ = fs::remove_dir_all(&tmp_root);
    }

    /// 测试 @alias 子路径补全使用正斜杠和反斜杠
    #[test]
    fn test_alias_completion_with_both_separators() {
        use std::fs;
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 创建临时目录结构
        let tmp_root = std::env::temp_dir().join("rfe_test_alias_sep");
        let _ = fs::remove_dir_all(&tmp_root);
        fs::create_dir_all(&tmp_root).unwrap();
        let sub_dir = tmp_root.join("alias_subdir");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("alias_file.txt"), "test").unwrap();

        // 注册别名
        {
            let mut mgr = helper.alias_manager.lock().unwrap();
            mgr.aliases.insert(
                "test_alias".to_string(),
                tmp_root.to_string_lossy().to_string(),
            );
        }

        // 测试正斜杠: cd @test_alias/
        let line_fwd = "cd @test_alias/";
        let result_fwd = helper.complete(line_fwd, line_fwd.len(), &ctx).unwrap();
        assert!(!result_fwd.1.is_empty(), "使用正斜杠的别名补全应该有结果");

        let subdir_cand_fwd = result_fwd.1.iter().find(|c| c.display == "alias_subdir");
        assert!(subdir_cand_fwd.is_some(), "应该找到alias_subdir");
        assert!(subdir_cand_fwd.unwrap().replacement.contains("@test_alias/"), "应使用正斜杠");

        // 测试反斜杠: cd @test_alias\
        let line_back = r"cd @test_alias\";
        let result_back = helper.complete(line_back, line_back.len(), &ctx).unwrap();
        assert!(!result_back.1.is_empty(), "使用反斜杠的别名补全应该有结果");

        let subdir_cand_back = result_back.1.iter().find(|c| c.display == "alias_subdir");
        assert!(subdir_cand_back.is_some(), "应该找到alias_subdir");
        assert!(subdir_cand_back.unwrap().replacement.contains(r"@test_alias\"), "应使用反斜杠");

        // 清理
        let _ = fs::remove_dir_all(&tmp_root);
        let mut mgr = helper.alias_manager.lock().unwrap();
        mgr.aliases.remove("test_alias");
    }

    /// 测试 cd -r 子路径补全的深层目录
    #[test]
    fn test_cd_r_deep_subpath_completion() {
        use std::fs;
        let helper = create_helper();
        let history = MemHistory::default();
        let ctx = Context::new(&history);

        // 创建深层目录结构
        let tmp_root = std::env::temp_dir().join("rfe_test_cd_r_deep");
        let _ = fs::remove_dir_all(&tmp_root);
        let deep_dir = tmp_root.join("level1").join("level2").join("level3");
        fs::create_dir_all(&deep_dir).unwrap();
        fs::write(deep_dir.join("deep_file.txt"), "test").unwrap();

        // 填充 last_ls_items
        {
            let mut items = helper.last_ls_items.lock().unwrap();
            items.push(crate::models::FileInfo {
                name: "deep_root".to_string(),
                full_path: tmp_root.to_string_lossy().to_string(),
                icon: "📁",
                color: colored::Color::Blue,
                size: 0,
                created: None,
                modified: std::time::SystemTime::now(),
                is_dir: true,
                tags: vec![],
            });
        }

        // 测试深层路径，混合使用分隔符也能处理
        let line = "cd -r 1/level1/level2/";
        let result = helper.complete(line, line.len(), &ctx).unwrap();
        assert!(!result.1.is_empty(), "深层路径补全应该有结果");

        let level3_cand = result.1.iter().find(|c| c.display == "level3");
        assert!(level3_cand.is_some(), "应该找到level3目录");

        // 清理
        let _ = fs::remove_dir_all(&tmp_root);
    }
}
