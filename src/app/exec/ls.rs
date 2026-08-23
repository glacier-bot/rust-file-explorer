//! ls 命令执行（列出目录内容，含正则/标签过滤）

use super::{ExecContext, ExecResult};
use crate::app::pipeline::CommandResult;

/// 执行 ls 命令（列出目录内容，含正则/标签过滤）
pub(crate) fn exec_ls(ctx: &ExecContext<'_>) -> ExecResult {
    let ExecContext { parts, alias_manager, tag_manager, last_ls_items, .. } = *ctx;

    let mut all = false;
    let mut long = false;
    let mut re = false;
    let mut re_insensitive = false;
    let mut show_tags = false;
    let mut recursive = false;
    let mut path_parts: Vec<String> = Vec::new();
    let mut tag_pattern_strs: Vec<String> = Vec::new();

    let mut i = 1;
    while i < parts.len() {
        match parts[i].as_str() {
            "-a" | "--all" => all = true,
            "-l" | "--long" => long = true,
            "-la" | "-al" => {
                all = true;
                long = true;
            }
            "--re" => re = true,
            "--re-deep" => {
                re = true;
                recursive = true;
            }
            "--re-insensitive" => re_insensitive = true,
            "--xcaps" => re_insensitive = true,
            "-tag" | "--tags" => show_tags = true,
            "-t" | "--tag" => {
                if i + 1 < parts.len() {
                    if parts[i + 1] == "--deep" && i + 2 < parts.len() {
                        recursive = true;
                        tag_pattern_strs.push(parts[i + 2].clone());
                        i += 2;
                    } else {
                        tag_pattern_strs.push(parts[i + 1].clone());
                        i += 1;
                    }
                } else {
                    return Err(
                        "Tag query parameter requires a pattern, usage: ls -t <tag_regex>"
                            .into(),
                    );
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

    let mut tag_patterns = Vec::new();
    for pattern_str in tag_pattern_strs {
        match regex::Regex::new(&pattern_str) {
            Ok(re) => tag_patterns.push(re),
            Err(e) => return Err(format!("Invalid tag regex: {}", e).into()),
        }
    }

    let (display, raw, items) = crate::commands::ls::cmd_ls(
        all,
        long,
        re,
        re_insensitive,
        show_tags,
        recursive,
        path.as_deref(),
        &tag_manager.lock().unwrap(),
        &tag_patterns,
    )?;
    *last_ls_items.lock().unwrap() = items;
    Ok((CommandResult::Normal(false), display, raw, None))
}
