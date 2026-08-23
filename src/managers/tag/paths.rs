//! 标签键的路径归一化工具
//! 负责 canonicalize、.index 占位符词法绝对化与 UNC 路径迁移

use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// 归一化标签键路径：存在的路径走 canonicalize；不存在的 .index 占位符走词法绝对化
pub(super) fn normalize_path(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() && is_index_placeholder(&path_buf) {
        return lexical_absolute(&path_buf);
    }
    let abs_path = fs::canonicalize(&path_buf)?;
    let mut path_str = abs_path.to_string_lossy().to_string();

    if cfg!(windows) {
        if path_str.starts_with("\\\\?\\UNC\\") {
            path_str = format!("\\\\{}", &path_str[8..]);
        } else if path_str.starts_with("\\\\?\\") {
            path_str = path_str[4..].to_string();
        }
    }

    Ok(path_str)
}

/// 判断路径是否为 .index 占位符（目录标签约定）
pub(super) fn is_index_placeholder(path: &Path) -> bool {
    matches!(path.file_name(), Some(name) if name == ".index")
}

/// 词法绝对化：不依赖文件实际存在，跳过 . 分量并向上弹出 .. 分量
fn lexical_absolute(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };

    let mut normalized = PathBuf::new();
    for component in abs.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }

    Ok(normalized.to_string_lossy().to_string())
}

/// 将 UNC 形式路径转换为常规形式
fn convert_unc_path_to_normal(path: &str) -> String {
    let mut path_str = path.to_string();

    if cfg!(windows) {
        if path_str.starts_with("\\\\?\\UNC\\") {
            path_str = format!("\\\\{}", &path_str[8..]);
        } else if path_str.starts_with("UNC\\") {
            path_str = format!("\\\\{}", &path_str[4..]);
        } else if path_str.starts_with("\\\\?\\") {
            path_str = path_str[4..].to_string();
        }
    }

    path_str
}

/// 迁移历史数据中的 UNC 形式键，返回是否发生迁移
pub(super) fn migrate_unc_paths(tags: &mut HashMap<String, Vec<String>>) -> bool {
    let mut need_migrate = false;
    let mut new_tags = HashMap::new();

    for (path, tag_list) in tags.drain() {
        let new_path = convert_unc_path_to_normal(&path);
        if new_path != path {
            need_migrate = true;
        }
        new_tags.insert(new_path, tag_list);
    }

    *tags = new_tags;
    need_migrate
}
