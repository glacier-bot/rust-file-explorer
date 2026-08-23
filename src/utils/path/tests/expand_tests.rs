use crate::utils::path::*;

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
fn test_expand_placeholder_with_subpath_forward_slash() {
    let r = expand_pop_placeholders("cp {}/test.txt /tmp/", "C:\\Users\\q\\project");
    assert_eq!(r.expanded, "cp C:\\Users\\q\\project/test.txt /tmp/");
    assert_eq!(r.total_replacements, 1);
}

#[test]
fn test_expand_placeholder_with_subpath_backslash() {
    let r = expand_pop_placeholders("cp {}\\test.txt /tmp/", "C:\\Users\\q\\project");
    assert_eq!(r.expanded, "cp C:\\Users\\q\\project\\test.txt /tmp/");
    assert_eq!(r.total_replacements, 1);
}

#[test]
fn test_expand_placeholder_with_pop_and_subpath() {
    let r = expand_pop_placeholders("open {}.pop/test.txt", "C:\\Users\\q\\project\\src");
    assert_eq!(r.expanded, "open C:\\Users\\q\\project/test.txt");
    assert_eq!(r.total_replacements, 1);
    assert_eq!(r.actual_pops, 1);
}

#[test]
fn test_expand_placeholder_with_multiple_dots_and_subpath() {
    let r = expand_pop_placeholders("ls {}../test.txt", "C:\\Users\\q\\project\\src");
    assert_eq!(r.expanded, "ls C:\\Users\\q/test.txt");
    assert_eq!(r.total_replacements, 1);
    assert_eq!(r.actual_pops, 2);
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
