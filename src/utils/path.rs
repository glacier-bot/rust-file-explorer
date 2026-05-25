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

#[derive(Debug, PartialEq)]
pub struct ExpandResult {
    pub expanded: String,
    pub reached_boundary: bool,
    pub actual_pops: usize,
    pub total_replacements: usize,
}

pub fn expand_pop_placeholders(cmd: &str, previous_raw_data: &str) -> ExpandResult {
    let mut result = String::with_capacity(cmd.len());
    let chars: Vec<char> = cmd.chars().collect();
    let mut i = 0;
    let mut reached_boundary = false;
    let mut last_actual_pops = 0;
    let mut total_replacements = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '}' {
            let mut j = i + 2;
            let mut pop_count = 0;

            while j < chars.len() {
                if chars[j] == '.' {
                    if j + 3 <= chars.len()
                        && chars[j + 1] == 'p'
                        && chars[j + 2] == 'o'
                        && chars[j + 3] == 'p'
                    {
                        pop_count += 1;
                        j += 4;
                    } else {
                        pop_count += 1;
                        j += 1;
                    }
                } else {
                    break;
                }
            }

            let pop_result = pop_path(previous_raw_data, pop_count);
            if pop_result.reached_boundary {
                reached_boundary = true;
            }
            last_actual_pops = pop_result.actual_pops;
            result.push_str(&pop_result.path);
            total_replacements += 1;
            i = j;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    ExpandResult {
        expanded: result,
        reached_boundary,
        actual_pops: last_actual_pops,
        total_replacements,
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

    #[test]
    fn test_pop_path_returns_pop_result_struct() {
        let r = pop_path("C:\\a\\b", 1);
        assert_eq!(
            r,
            PopResult {
                path: "C:\\a".to_string(),
                actual_pops: 1,
                reached_boundary: false,
            }
        );
    }

    #[test]
    fn test_pop_path_empty_string_zero_pops_not_boundary() {
        let r = pop_path("", 0);
        assert_eq!(r.actual_pops, 0);
        assert!(!r.reached_boundary);
        assert_eq!(r.path, "");
    }

    #[test]
    fn test_pop_path_actual_pops_never_exceeds_request() {
        let r = pop_path("C:\\Users\\q\\Desktop", 2);
        assert!(r.actual_pops <= 2);
    }

    #[test]
    fn test_pop_path_mixed_slashes() {
        let r = pop_path("C:\\Users/q\\Desktop/folder", 2);
        assert_eq!(r.actual_pops, 2);
        assert!(!r.reached_boundary);
    }

    #[test]
    fn test_expand_no_placeholder() {
        let r = expand_pop_placeholders("ls -a", "C:\\Users\\q");
        assert_eq!(r.expanded, "ls -a");
        assert_eq!(r.total_replacements, 0);
        assert!(!r.reached_boundary);
        assert_eq!(r.actual_pops, 0);
    }

    #[test]
    fn test_expand_single_placeholder_no_pop() {
        let r = expand_pop_placeholders("cd {}", "C:\\Users\\q\\Desktop");
        assert_eq!(r.expanded, "cd C:\\Users\\q\\Desktop");
        assert_eq!(r.total_replacements, 1);
        assert!(!r.reached_boundary);
        assert_eq!(r.actual_pops, 0);
    }

    #[test]
    fn test_expand_single_placeholder_one_pop() {
        let r = expand_pop_placeholders("cd {}.pop", "C:\\Users\\q\\Desktop");
        assert_eq!(r.expanded, "cd C:\\Users\\q");
        assert_eq!(r.total_replacements, 1);
        assert!(!r.reached_boundary);
        assert_eq!(r.actual_pops, 1);
    }

    #[test]
    fn test_expand_single_placeholder_one_dot() {
        let r = expand_pop_placeholders("cd {}.", "C:\\Users\\q\\Desktop");
        assert_eq!(r.expanded, "cd C:\\Users\\q");
        assert_eq!(r.total_replacements, 1);
        assert!(!r.reached_boundary);
        assert_eq!(r.actual_pops, 1);
    }

    #[test]
    fn test_expand_pop_chain_two_pops() {
        let r = expand_pop_placeholders("cd {}.pop.pop", "C:\\Users\\q\\Desktop\\rust-file-explorer");
        assert_eq!(r.expanded, "cd C:\\Users\\q");
        assert_eq!(r.total_replacements, 1);
        assert!(!r.reached_boundary);
        assert_eq!(r.actual_pops, 2);
    }

    #[test]
    fn test_expand_pop_chain_three_pops() {
        let r = expand_pop_placeholders("cd {}.pop.pop.pop", "C:\\Users\\q\\Desktop\\rust-file-explorer");
        assert_eq!(r.expanded, "cd C:\\Users");
        assert_eq!(r.total_replacements, 1);
        assert!(!r.reached_boundary);
        assert_eq!(r.actual_pops, 3);
    }

    #[test]
    fn test_expand_single_placeholder_three_dots() {
        let r = expand_pop_placeholders("cd {}...", "C:\\Users\\q\\Desktop\\rust-file-explorer");
        assert_eq!(r.expanded, "cd C:\\Users");
        assert_eq!(r.total_replacements, 1);
        assert_eq!(r.actual_pops, 3);
        assert!(!r.reached_boundary);
    }

    #[test]
    fn test_expand_placeholder_reached_boundary() {
        let r = expand_pop_placeholders("cd {}.........", "C:\\Users");
        assert!(r.reached_boundary);
        assert_eq!(r.total_replacements, 1);
    }

    #[test]
    fn test_expand_placeholder_empty_input() {
        let r = expand_pop_placeholders("cd {}.", "");
        assert_eq!(r.expanded, "cd ");
        assert!(r.reached_boundary);
        assert_eq!(r.total_replacements, 1);
    }

    #[test]
    fn test_expand_multiple_placeholders() {
        let r = expand_pop_placeholders("cp {} {}.", "C:\\Users\\q\\Desktop");
        assert_eq!(r.expanded, "cp C:\\Users\\q\\Desktop C:\\Users\\q");
        assert_eq!(r.total_replacements, 2);
        assert!(!r.reached_boundary);
    }

    #[test]
    fn test_expand_dot_chain_does_not_leak_into_next_arg() {
        let r = expand_pop_placeholders("mv {}. file.txt", "C:\\Users\\q\\Desktop");
        assert_eq!(r.expanded, "mv C:\\Users\\q file.txt");
        assert_eq!(r.total_replacements, 1);
    }

    #[test]
    fn test_expand_consecutive_placeholders() {
        let r = expand_pop_placeholders("echo {}{}", "C:\\foo\\bar");
        assert_eq!(r.expanded, "echo C:\\foo\\barC:\\foo\\bar");
        assert_eq!(r.total_replacements, 2);
        assert!(!r.reached_boundary);
    }

    #[test]
    fn test_expand_no_substitution_means_no_pops_counted() {
        let r = expand_pop_placeholders("ls", "C:\\Users\\q");
        assert_eq!(r.actual_pops, 0);
        assert!(!r.reached_boundary);
        assert_eq!(r.total_replacements, 0);
    }

    #[test]
    fn test_expand_terminates_when_replacement_contains_braces_text() {
        let r = expand_pop_placeholders("echo {}", "no_braces_here");
        assert_eq!(r.expanded, "echo no_braces_here");
        assert_eq!(r.total_replacements, 1);
    }

    #[test]
    fn test_expand_terminates_with_braces_in_previous_data() {
        let r = expand_pop_placeholders("echo {}", "value_{}_x");
        assert_eq!(r.expanded, "echo value_{}_x");
        assert_eq!(r.total_replacements, 1);
    }

    #[test]
    fn test_expand_unicode_path_in_previous_data() {
        let r = expand_pop_placeholders("cd {}", "C:\\用户\\桌面");
        assert_eq!(r.expanded, "cd C:\\用户\\桌面");
        assert_eq!(r.total_replacements, 1);
    }

    #[test]
    fn test_expand_unicode_command() {
        let r = expand_pop_placeholders("查看 {}", "/home/项目");
        assert_eq!(r.expanded, "查看 /home/项目");
        assert_eq!(r.total_replacements, 1);
    }

    #[test]
    fn test_expand_unix_style_with_pops() {
        let r = expand_pop_placeholders("cd {}..", "/home/user/project");
        assert_eq!(r.expanded, "cd /home");
        assert_eq!(r.actual_pops, 2);
        assert!(!r.reached_boundary);
    }

    #[test]
    fn test_expand_pops_count_reflects_last_placeholder() {
        let r = expand_pop_placeholders("a {} b {}.", "C:\\a\\b\\c");
        assert_eq!(r.actual_pops, 1);
        assert_eq!(r.total_replacements, 2);
    }

    #[test]
    fn test_expand_lone_open_brace_is_kept() {
        let r = expand_pop_placeholders("echo { hello", "/data");
        assert_eq!(r.expanded, "echo { hello");
        assert_eq!(r.total_replacements, 0);
    }

    #[test]
    fn test_expand_braces_with_content_inside_not_replaced() {
        let r = expand_pop_placeholders("echo {x}", "/data");
        assert_eq!(r.expanded, "echo {x}");
        assert_eq!(r.total_replacements, 0);
    }

    #[test]
    fn test_expand_reached_boundary_with_multiple_placeholders_any() {
        let r = expand_pop_placeholders("cp {} {}..........", "C:\\Users");
        assert!(r.reached_boundary);
        assert_eq!(r.total_replacements, 2);
    }
}
