//! cd 命令执行（切换目录，含 -tag/-sel/-r 参数处理）

use super::{ExecContext, ExecResult};
use crate::app::pipeline::CommandResult;
use crate::resolver::resolve_line_path;
use colored::Colorize;

/// 执行 cd 命令（切换目录，含 -tag/-sel/-r 参数处理）
pub(crate) fn exec_cd(ctx: &ExecContext<'_>) -> ExecResult {
    let ExecContext {
        input,
        input_data,
        parts,
        alias_manager,
        tag_manager,
        last_ls_items,
        previous_dir,
    } = *ctx;

    let has_shell_ops = input.contains('&')
        || input.contains('|')
        || input.contains(';')
        || input.contains('>')
        || input.contains('<')
        || input.contains("&&")
        || input.contains("||");

    if has_shell_ops {
        match crate::commands::shell::cmd_shell(input) {
            Ok((display, raw, new_prev_dir)) => {
                Ok((CommandResult::Normal(false), display, raw, new_prev_dir))
            }
            Err(e) => Err(e),
        }
    } else {
        let mut is_idx = false;
        let mut idx_tag: Option<String> = None;
        let mut path_parts: Vec<String> = Vec::new();
        let mut selection: Option<usize> = None;

        let mut i = 1;
        while i < parts.len() {
            match parts[i].as_str() {
                "-tag" => {
                    is_idx = true;
                    if i + 1 < parts.len() {
                        idx_tag = Some(parts[i + 1].clone());
                        i += 1;
                    }
                }
                "-sel" => {
                    if i + 1 < parts.len() {
                        if let Ok(n) = parts[i + 1].parse::<usize>() {
                            selection = Some(n);
                        }
                        i += 1;
                    }
                }
                "-r" => {
                    if i + 1 < parts.len() {
                        let target_path = resolve_line_path(&parts[i + 1], last_ls_items)?;
                        let target_path_buf = std::path::PathBuf::from(&target_path);

                        if !target_path_buf.exists() {
                            return Err(format!(
                                "Path does not exist: {}",
                                target_path
                            )
                            .into());
                        }
                        if !target_path_buf.is_dir() {
                            return Err(format!(
                                "'{}' is not a directory",
                                target_path
                            )
                            .into());
                        }

                        let current_dir = std::env::current_dir()?;
                        std::env::set_current_dir(&target_path_buf)?;
                        let new_prev_dir = if target_path_buf != current_dir {
                            Some(current_dir.display().to_string())
                        } else {
                            None
                        };
                        let display = format!(
                            "{} {}",
                            "Changed to:".green(),
                            target_path.cyan()
                        );
                        return Ok((
                            CommandResult::Normal(false),
                            display,
                            target_path,
                            new_prev_dir,
                        ));
                    } else {
                        return Err("Usage: cd -r <line_number>[/sub_path]".into());
                    }
                }
                p => {
                    let resolved = alias_manager.lock().unwrap().resolve_path(p);
                    path_parts.push(resolved);
                }
            }
            i += 1;
        }

        let path = if path_parts.is_empty() {
            None
        } else {
            Some(path_parts.join(" "))
        };

        if is_idx {
            match crate::commands::cd::cmd_cd(
                None,
                previous_dir,
                true,
                idx_tag.as_deref(),
                Some(&tag_manager.lock().unwrap()),
                selection,
            )? {
                crate::commands::cd::CdResult::Success(display, raw, new_prev) => {
                    Ok((CommandResult::Normal(false), display, raw, new_prev))
                }
                crate::commands::cd::CdResult::NeedSelection(items) => Ok((
                    CommandResult::NeedCdSelection(items),
                    String::new(),
                    String::new(),
                    None,
                )),
            }
        } else {
            let path = if path.is_some() {
                path
            } else if !input_data.is_empty() {
                Some(input_data.to_string())
            } else {
                None
            };
            match crate::commands::cd::cmd_cd(
                path.as_deref(),
                previous_dir,
                false,
                None,
                None,
                None,
            )? {
                crate::commands::cd::CdResult::Success(display, raw, new_prev) => {
                    Ok((CommandResult::Normal(false), display, raw, new_prev))
                }
                crate::commands::cd::CdResult::NeedSelection(_) => {
                    Err("Unexpected error".into())
                }
            }
        }
    }
}
