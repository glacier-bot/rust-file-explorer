use crate::utils::path::*;

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
