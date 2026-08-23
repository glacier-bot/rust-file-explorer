//! cpf / mv 命令执行（剪贴板复制与文件移动/复制，支持 -r 行号引用）

use super::{ExecContext, ExecResult};
use crate::app::pipeline::CommandResult;
use crate::resolver::resolve_line_path;

/// 执行 cpf 命令（复制文件路径到剪贴板）
pub(crate) fn exec_cpf(ctx: &ExecContext<'_>) -> ExecResult {
    let ExecContext { parts, input_data, alias_manager, last_ls_items, .. } = *ctx;

    let mut path_parts: Vec<String> = Vec::new();
    let mut i = 1;
    while i < parts.len() {
        if parts[i] == "-r" {
            if i + 1 >= parts.len() {
                return Err("Usage: cpf -r <line_number>[/path]".into());
            }
            let resolved = resolve_line_path(&parts[i + 1], last_ls_items)?;
            path_parts.push(resolved);
            i += 2;
        } else {
            let resolved = alias_manager.lock().unwrap().resolve_path(&parts[i]);
            path_parts.push(resolved);
            i += 1;
        }
    }

    let path = if path_parts.is_empty() {
        if !input_data.is_empty() {
            Some(input_data.to_string())
        } else {
            None
        }
    } else {
        Some(path_parts.join(" "))
    }
    .ok_or("Usage: cpf <file> or cpf -r <line_number>[/path]")?;

    let (display, raw) = crate::commands::clipboard::cmd_cpf(&path)?;
    Ok((CommandResult::Normal(false), display, raw, None))
}

/// 执行 mv 命令（移动/复制文件，支持 -r 行号引用）
pub(crate) fn exec_mv(ctx: &ExecContext<'_>) -> ExecResult {
    let ExecContext { parts, alias_manager, last_ls_items, .. } = *ctx;

    let mut path_parts: Vec<String> = Vec::new();
    let mut copy = false;

    let mut i = 1;
    while i < parts.len() {
        match parts[i].as_str() {
            "--cp" => {
                copy = true;
                i += 1;
            }
            "-r" => {
                if i + 1 >= parts.len() {
                    return Err("Missing path after -r parameter".into());
                }
                let resolved = resolve_line_path(&parts[i + 1], last_ls_items)?;
                path_parts.push(resolved);
                i += 2;
            }
            part => {
                let resolved = alias_manager.lock().unwrap().resolve_path(part);
                path_parts.push(resolved);
                i += 1;
            }
        }
    }

    if path_parts.len() < 2 {
        return Err(
            "Usage: mv <source_path> <destination_path> [--cp] or mv -r <source_line> <destination> or mv <source> -r <destination_line>"
                .into(),
        );
    }

    let destination = path_parts.pop().unwrap();
    let source = path_parts.join(" ");

    let (display, raw) = crate::commands::mv::cmd_mv(&source, &destination, copy)?;
    Ok((CommandResult::Normal(false), display, raw, None))
}
