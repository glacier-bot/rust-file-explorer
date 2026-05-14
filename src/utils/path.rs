use std::fs;
use std::path::PathBuf;

pub fn is_hidden(path: &PathBuf) -> bool {
    #[cfg(unix)]
    {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        match fs::metadata(path) {
            Ok(meta) => (meta.file_attributes() & 2) != 0,
            Err(_) => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PopResult {
    pub path: String,
    pub actual_pops: usize,
    pub reached_boundary: bool,
}

pub fn pop_path(path: &str, pop_count: usize) -> PopResult {
    if path.is_empty() {
        return PopResult {
            path: String::new(),
            actual_pops: 0,
            reached_boundary: pop_count > 0,
        };
    }

    let mut path_buf = PathBuf::from(path);
    let mut actual_pops = 0;
    let mut reached_boundary = false;

    for _ in 0..pop_count {
        let current_path = path_buf.clone();
        if let Some(parent) = path_buf.parent() {
            let parent_str = parent.to_string_lossy();
            let current_str = current_path.to_string_lossy();
            
            if parent_str == current_str {
                reached_boundary = true;
                break;
            }
            
            path_buf = parent.to_path_buf();
            actual_pops += 1;
        } else {
            reached_boundary = true;
            break;
        }
    }

    PopResult {
        path: path_buf.to_string_lossy().to_string(),
        actual_pops,
        reached_boundary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pop_path_normal_single_pop() {
        let result = pop_path("C:\\Users\\q\\Desktop\\rust-file-explorer", 1);
        assert_eq!(result.actual_pops, 1);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "C:\\Users\\q\\Desktop");
    }

    #[test]
    fn test_pop_path_multiple_pops() {
        let result = pop_path("C:\\Users\\q\\Desktop\\rust-file-explorer", 3);
        assert_eq!(result.actual_pops, 3);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "C:\\Users");
    }

    #[test]
    fn test_pop_path_zero_pops() {
        let result = pop_path("C:\\Users\\q\\Desktop", 0);
        assert_eq!(result.actual_pops, 0);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "C:\\Users\\q\\Desktop");
    }

    #[test]
    fn test_pop_path_boundary() {
        let result = pop_path("C:\\", 1);
        assert!(result.reached_boundary);
        assert_eq!(result.path, "C:\\");
    }

    #[test]
    fn test_pop_path_exceed_boundary() {
        let result = pop_path("C:\\Users\\q", 10);
        assert!(result.actual_pops >= 2);
        assert!(result.reached_boundary);
    }

    #[test]
    fn test_pop_path_empty_string() {
        let result = pop_path("", 1);
        assert_eq!(result.actual_pops, 0);
        assert!(result.reached_boundary);
        assert_eq!(result.path, "");
    }

    #[test]
    fn test_pop_path_relative() {
        let result = pop_path("folder/subfolder/file.txt", 2);
        assert_eq!(result.actual_pops, 2);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "folder");
    }

    #[test]
    fn test_pop_path_just_filename() {
        let result = pop_path("file.txt", 1);
        assert_eq!(result.actual_pops, 1);
        assert_eq!(result.path, "");
    }

    #[test]
    fn test_pop_path_just_filename_multiple() {
        let result = pop_path("file.txt", 5);
        assert!(result.actual_pops >= 1);
        assert!(result.reached_boundary);
        assert_eq!(result.path, "");
    }

    #[test]
    fn test_pop_path_unc() {
        let result = pop_path("\\\\server\\share\\folder", 1);
        assert_eq!(result.actual_pops, 1);
        assert!(!result.reached_boundary);
        assert!(result.path.contains("share"));
    }

    #[test]
    fn test_pop_path_with_special_chars() {
        let result = pop_path("C:\\Users\\q\\My Docs\\file.txt", 2);
        assert_eq!(result.actual_pops, 2);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "C:\\Users\\q");
    }

    #[test]
    fn test_pop_path_root_exact() {
        let result = pop_path("C:\\Users\\q", 2);
        assert_eq!(result.actual_pops, 2);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "C:\\");
    }

    #[test]
    fn test_pop_path_root_plus_one() {
        let result = pop_path("C:\\Users\\q", 3);
        assert_eq!(result.actual_pops, 2);
        assert!(result.reached_boundary);
        assert_eq!(result.path, "C:\\");
    }

    #[test]
    fn test_pop_path_single_component() {
        let result = pop_path("test", 1);
        assert_eq!(result.actual_pops, 1);
        assert_eq!(result.path, "");
    }

    #[test]
    fn test_pop_path_single_component_twice() {
        let result = pop_path("test", 2);
        assert!(result.actual_pops >= 1);
        assert!(result.reached_boundary);
        assert_eq!(result.path, "");
    }

    #[test]
    fn test_pop_path_trailing_slash() {
        let result = pop_path("C:\\Users\\q\\", 1);
        assert_eq!(result.actual_pops, 1);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "C:\\Users");
    }

    #[test]
    fn test_pop_path_forward_slashes() {
        let result = pop_path("/home/user/project", 2);
        assert_eq!(result.actual_pops, 2);
        assert!(!result.reached_boundary);
        assert_eq!(result.path, "/home");
    }

    #[test]
    fn test_pop_path_unix_root() {
        let result = pop_path("/", 1);
        assert!(result.reached_boundary);
        assert_eq!(result.path, "/");
    }

    #[test]
    fn test_pop_path_unix_root_plus() {
        let result = pop_path("/home", 5);
        assert!(result.reached_boundary);
    }
}
