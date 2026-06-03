//! 行号路径解析模块
//! 负责将行号路径（如 "2/src/main.rs"）解析为实际的文件系统路径

use crate::models::FileInfo;
use std::sync::{Arc, Mutex};

/// 解析行号路径，将类似 "2/src/main.rs" 转换为实际的绝对路径
///
/// # Arguments
/// * `path_part` - 行号路径字符串，支持 Unix 和 Windows 路径分隔符
/// * `last_ls_items` - 上一次 ls 命令的结果列表
///
/// # Returns
/// * 解析后的完整路径字符串
///
/// # Errors
/// * 路径部分为空时返回错误
/// * 行号无效（非数字、超出范围）时返回错误
#[cfg_attr(test, allow(dead_code))]
pub fn resolve_line_path(
    path_part: &str,
    last_ls_items: &Arc<Mutex<Vec<FileInfo>>>,
) -> Result<String, Box<dyn std::error::Error>> {
    // 分割行号和子路径（支持Unix和Windows路径分隔符）
    let (line_num_str, sub_path) =
        if let Some(slash_pos) = path_part.find(|c: char| c == '/' || c == '\\') {
            (&path_part[..slash_pos], &path_part[slash_pos + 1..])
        } else {
            (path_part, "")
        };

    let line_num = line_num_str
        .parse::<usize>()
        .map_err(|_| "Invalid line number")?;
    let items = last_ls_items.lock().unwrap();
    if line_num < 1 || line_num > items.len() {
        return Err(format!(
            "Line number {} out of range (1-{})",
            line_num,
            items.len()
        )
        .into());
    }
    let item = &items[line_num - 1];

    // 拼接完整路径：使用字符串拼接以保持原始分隔符风格
    let full_path = if sub_path.is_empty() {
        // 当子路径为空时，检查原始输入是否以分隔符结尾
        // 如果是，则保留尾部分隔符（与原始 PathBuf::join 行为一致）
        if path_part.ends_with(|c: char| c == '/' || c == '\\') {
            format!("{}/", item.full_path)
        } else {
            item.full_path.clone()
        }
    } else {
        // 检测 base 路径使用的分隔符风格
        let base = &item.full_path;
        let sep = if base.contains('\\') { "\\" } else { "/" };
        let base_trimmed = base.trim_end_matches(|c| c == '/' || c == '\\');
        let sub_trimmed = sub_path.trim_start_matches(|c| c == '/' || c == '\\');
        format!("{}{}{}", base_trimmed, sep, sub_trimmed)
    };

    Ok(full_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FileInfo;
    use colored::Color;
    use std::sync::{Arc, Mutex};

    fn create_test_file_info(name: &str, full_path: &str, is_dir: bool) -> FileInfo {
        FileInfo {
            name: name.to_string(),
            full_path: full_path.to_string(),
            icon: if is_dir { "📁" } else { "📄" },
            color: if is_dir {
                Color::Blue
            } else {
                Color::White
            },
            size: 0,
            created: None,
            modified: std::time::SystemTime::now(),
            is_dir,
            tags: vec![],
        }
    }

    #[test]
    fn test_resolve_line_path_valid_line_number() {
        let last_ls_items = Arc::new(Mutex::new(vec![
            create_test_file_info("test_dir", "/test/path/test_dir", true),
            create_test_file_info("test_file.txt", "/test/path/test_file.txt", false),
        ]));

        let result = resolve_line_path("1", &last_ls_items).unwrap();
        assert_eq!(result, "/test/path/test_dir");

        let result = resolve_line_path("2", &last_ls_items).unwrap();
        assert_eq!(result, "/test/path/test_file.txt");
    }

    #[test]
    fn test_resolve_line_path_out_of_range() {
        let last_ls_items = Arc::new(Mutex::new(vec![create_test_file_info(
            "test_file",
            "/test/path/test_file",
            false,
        )]));

        let result = resolve_line_path("0", &last_ls_items);
        assert!(result.is_err());

        let result = resolve_line_path("2", &last_ls_items);
        assert!(result.is_err());

        let result = resolve_line_path("100", &last_ls_items);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_line_path_invalid_number() {
        let last_ls_items = Arc::new(Mutex::new(vec![create_test_file_info(
            "test",
            "/test",
            true,
        )]));

        let result = resolve_line_path("abc", &last_ls_items);
        assert!(result.is_err());

        let result = resolve_line_path("", &last_ls_items);
        assert!(result.is_err());

        let result = resolve_line_path("-1", &last_ls_items);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_line_path_with_sub_path_forward_slash() {
        let last_ls_items = Arc::new(Mutex::new(vec![create_test_file_info(
            "project",
            "/home/user/project",
            true,
        )]));

        let result = resolve_line_path("1/src/main.rs", &last_ls_items).unwrap();
        assert_eq!(result, "/home/user/project/src/main.rs");
    }

    #[test]
    fn test_resolve_line_path_with_sub_path_backslash() {
        let last_ls_items = Arc::new(Mutex::new(vec![create_test_file_info(
            "project",
            "C:\\Users\\user\\project",
            true,
        )]));

        let result = resolve_line_path("1\\src\\main.rs", &last_ls_items).unwrap();
        assert_eq!(result, "C:\\Users\\user\\project\\src\\main.rs");
    }

    #[test]
    fn test_resolve_line_path_with_empty_sub_path() {
        let last_ls_items = Arc::new(Mutex::new(vec![create_test_file_info(
            "test",
            "/test/path",
            true,
        )]));

        let result = resolve_line_path("1/", &last_ls_items).unwrap();
        assert_eq!(result, "/test/path/");

        let result = resolve_line_path("1\\", &last_ls_items).unwrap();
        assert_eq!(result, "/test/path/");
    }

    #[test]
    fn test_resolve_line_path_empty_ls_items() {
        let last_ls_items = Arc::new(Mutex::new(Vec::new()));

        let result = resolve_line_path("1", &last_ls_items);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_line_path_deep_sub_path() {
        let last_ls_items = Arc::new(Mutex::new(vec![create_test_file_info(
            "root",
            "/root",
            true,
        )]));

        let result = resolve_line_path("1/a/b/c/d/e/f/g", &last_ls_items).unwrap();
        assert_eq!(result, "/root/a/b/c/d/e/f/g");
    }
}
