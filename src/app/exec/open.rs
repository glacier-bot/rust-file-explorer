//! open 命令执行（打开文件/目录，含 -tag/-sel/-r 参数处理）

use super::{ExecContext, ExecResult};
use crate::app::pipeline::CommandResult;
use crate::resolver::resolve_line_path;

/// 执行 open 命令（打开文件/目录，含 -tag/-sel/-r 参数处理）
pub(crate) fn exec_open(ctx: &ExecContext<'_>) -> ExecResult {
    let ExecContext { parts, input_data, alias_manager, tag_manager, last_ls_items, .. } = *ctx;

    let mut is_tag = false;
    let mut tag_value: Option<String> = None;
    let mut path_parts: Vec<String> = Vec::new();
    let mut selection: Option<usize> = None;
    let mut i = 1;
    while i < parts.len() {
        match parts[i].as_str() {
            "-tag" => {
                is_tag = true;
                if i + 1 < parts.len() {
                    tag_value = Some(parts[i + 1].clone());
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
                if i + 1 >= parts.len() {
                    return Err("Usage: open -r <line_number>[/path]".into());
                }
                let resolved = resolve_line_path(&parts[i + 1], last_ls_items)?;
                path_parts.push(resolved);
                i += 2;
            }
            p => {
                let resolved = alias_manager.lock().unwrap().resolve_path(p);
                path_parts.push(resolved);
            }
        }
        i += 1;
    }

    if is_tag {
        match crate::commands::open::cmd_open_tag(
            tag_value.as_deref(),
            Some(&tag_manager.lock().unwrap()),
            selection,
        )? {
            crate::commands::open::OpenResult::Success(display, raw) => {
                Ok((CommandResult::Normal(false), display, raw, None))
            }
            crate::commands::open::OpenResult::NeedSelection(items) => Ok((
                CommandResult::NeedOpenSelection(items),
                String::new(),
                String::new(),
                None,
            )),
        }
    } else {
        let path = if path_parts.is_empty() {
            if !input_data.is_empty() {
                Some(input_data.to_string())
            } else {
                None
            }
        } else {
            Some(path_parts.join(" "))
        }
        .ok_or("Usage: open <file> or open -r <line_number>[/path] or open -tag <tag>")?;

        let (display, raw) = crate::commands::open::cmd_open(&path)?;
        Ok((CommandResult::Normal(false), display, raw, None))
    }
}
