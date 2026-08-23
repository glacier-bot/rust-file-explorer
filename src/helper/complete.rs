//! RfeHelper 的 Completer 实现
//! 提供命令名、参数、子命令、标签与路径的补全入口

use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::Context;

use super::completion_helpers::{
    apply_quote_policy, check_in_quote, complete_alias_path, complete_line_number_path,
    complete_tag_command, is_after_closing_quote, is_before_closing_quote,
};
use super::RfeHelper;
use crate::completion::CompletionContext;

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

        // 检查引号相关状态（需要先计算，后面要用）
        let (in_quote, quote_char) = check_in_quote(line, pos);
        let cursor_before_closing_quote = is_before_closing_quote(line, pos);

        // cd -r <行号>/<子路径> 补全
        if let Some((start, mut candidates)) = complete_line_number_path(line, pos, &self.last_ls_items) {
            apply_quote_policy(&mut candidates, in_quote, quote_char, cursor_before_closing_quote);
            return Ok((start, candidates));
        }

        // 路径别名补全 - 支持 @alias/path 的层级补全
        if let Some((start, mut candidates)) = complete_alias_path(current_word, pos, &self.alias_manager) {
            apply_quote_policy(&mut candidates, in_quote, quote_char, cursor_before_closing_quote);
            return Ok((start, candidates));
        }

        // 标签补全：当命令是tag add/tag remove时补全标签名
        if let Some(result) = complete_tag_command(line, pos, current_word, &self.tag_manager) {
            return Ok(result);
        }

        // 检查光标是否紧邻闭合引号之后，这种情况路径已经完整，不提供路径补全
        if is_after_closing_quote(line, pos) {
            return Ok((pos, Vec::new()));
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

        let mut candidates = result.1;
        apply_quote_policy(&mut candidates, in_quote, quote_char, cursor_before_closing_quote);
        Ok((result.0, candidates))
    }
}
