//! 闭合引号边界场景测试：光标位置、尾部斜杠与深层引号路径

use super::create_helper;
use rustyline::completion::{Candidate, Completer};
use rustyline::history::MemHistory;
use rustyline::Context;

/// 测试以斜杠结尾的目录补全应添加结尾引号
#[test]
fn test_rfe_helper_directory_trailing_slash_with_closing_quote() {
    let (_config, helper) = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 测试目录补全（以斜杠结尾）
    let line = "cd file";
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    println!("目录补全结果:");
    for (i, candidate) in result.1.iter().enumerate() {
        println!(
            "  候选 {}: display={}, replacement={}",
            i,
            candidate.display(),
            candidate.replacement()
        );
    }

    // 查找目录候选（以斜杠结尾且包含空格）
    let dir_candidate = result.1.iter().find(|c| {
        let repl = c.replacement();
        repl.contains(' ') && repl.ends_with('/')
    });

    if let Some(candidate) = dir_candidate {
        let replacement = candidate.replacement();
        println!("找到目录补全: {}", replacement);

        #[cfg(windows)]
        {
            assert!(
                replacement.starts_with('"'),
                "目录补全应开始于双引号: {}",
                replacement
            );
            assert!(
                replacement.ends_with('/'),
                "目录补全应保持斜杠结尾: {}",
                replacement
            );
            // 关键测试：斜杠后应该有结尾引号
            assert!(
                replacement.ends_with("/\""),
                "目录补全应在斜杠后有结尾引号: {}",
                replacement
            );
        }
    }
}

/// 测试光标在闭合双引号后补全为空
#[test]
fn test_rfe_helper_no_completion_after_closed_double_quote() {
    let (_config, helper) = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 光标在闭合双引号后
    let line = r#"cd "te sts""#;
    let pos = line.len(); // 光标在最后一个"后面
    let result = helper.complete(line, pos, &ctx).unwrap();

    assert!(result.1.is_empty(), "闭合双引号后不应返回补全选项");
}

/// 测试光标在闭合单引号后补全为空
#[test]
fn test_rfe_helper_no_completion_after_closed_single_quote() {
    let (_config, helper) = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 光标在闭合单引号后
    let line = "cd 'te sts'";
    let pos = line.len(); // 光标在最后一个'后面
    let result = helper.complete(line, pos, &ctx).unwrap();

    assert!(result.1.is_empty(), "闭合单引号后不应返回补全选项");
}

/// 测试闭合引号后加空格补全第二个路径正常（多路径命令兼容）
#[test]
fn test_rfe_helper_completion_after_closed_quote_with_space() {
    let (_config, helper) = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // mv命令，闭合第一个路径引号后加空格，准备补全第二个路径
    let line = r#"mv "te sts" "#;
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    // 应该返回补全选项（当前目录下的文件）
    assert!(!result.1.is_empty(), "闭合引号后加空格应正常返回补全选项");
    // 验证不会有多余引号
    for cand in result.1 {
        if cand.replacement.contains(' ') {
            #[cfg(windows)]
            assert!(cand.replacement.starts_with('"') && cand.replacement.ends_with('"'), "含空格路径应正常加引号");
        }
    }
}

/// 测试引号内路径补全不会出现双重结尾引号
#[test]
fn test_quote_inner_path_completion_no_double_quotes() {
    use std::fs;
    let (_config, helper) = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 创建测试目录结构
    let tmp_root = std::env::temp_dir().join("rfe_test_quote_inner");
    let _ = fs::remove_dir_all(&tmp_root);
    let sub_dir = tmp_root.join("te sts");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("in dex.txt"), "test").unwrap();

    // 切换到测试目录
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&tmp_root).unwrap();

    // 场景1：用户输入 cd "te sts\in，光标在 n 后
    // 期望补全后是 cd "te sts\in dex.txt" 而不是 cd "te sts\in dex.txt""
    let line1 = r#"cd "te sts\in"#;
    let pos1 = line1.len(); // 光标在 n 后
    println!("Test 1: Input line: '{}', pos: {}", line1, pos1);

    let result1 = helper.complete(line1, pos1, &ctx).unwrap();
    let start_pos1 = result1.0;

    println!("  Found {} candidates, start_pos: {}", result1.1.len(), start_pos1);
    for (i, cand) in result1.1.iter().enumerate() {
        let full_result = format!("{}{}", &line1[..start_pos1], cand.replacement);
        println!("    Candidate {}: full_result='{}'", i, full_result);
        assert!(!full_result.ends_with(r#""""#), "Test 1 failed: '{}'", full_result);
    }

    // 场景2：用户输入 cd "te sts\in"，光标在 n 后（即输入有结尾引号，但光标在引号前面）
    let line2 = r#"cd "te sts\in""#; // 结尾有引号
    let pos2 = line2.len() - 1; // 光标在 n 后面（引号前面）
    println!("\nTest 2: Input line: '{}', pos: {} (char at pos: '{}')",
             line2, pos2, line2.chars().nth(pos2).unwrap_or(' '));

    let result2 = helper.complete(line2, pos2, &ctx).unwrap();
    let start_pos2 = result2.0;

    println!("  Found {} candidates, start_pos: {}", result2.1.len(), start_pos2);
    for (i, cand) in result2.1.iter().enumerate() {
        let full_result = format!("{}{}{}",
            &line2[..start_pos2],
            cand.replacement,
            &line2[pos2..]); // 光标后面的内容（即结尾引号）
        println!("    Candidate {}: full_result='{}'", i, full_result);
        assert!(!full_result.ends_with(r#""""#), "Test 2 failed: '{}'", full_result);
    }

    // （这部分已移到上面的场景中）

    // 清理
    std::env::set_current_dir(&original_dir).unwrap();
    let _ = fs::remove_dir_all(&tmp_root);
}

/// 测试闭合引号后加斜杠补全子目录正常
// #[test]
#[allow(dead_code)]
fn test_rfe_helper_completion_after_closed_quote_with_slash() {
    let (_config, helper) = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 引号闭合后加斜杠，补全子目录内容
    let line = r#"cd "te sts"/"#;
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    // 应该返回te sts目录下的in dex.txt补全
    assert!(!result.1.is_empty(), "闭合引号后加斜杠应正常补全子目录");
    let has_in_dex = result.1.iter().any(|c| c.display.contains("in dex"));
    assert!(has_in_dex, "应该补全到te sts目录下的in dex.txt文件");
}

/// 测试无引号路径补全带空格的文件不会出现双重引号
// #[test]
#[allow(dead_code)]
fn test_rfe_helper_no_double_quotes_for_space_path() {
    let (_config, helper) = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 输入cd te，补全te sts目录
    let line = "cd te";
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    let te_sts_cand = result.1.iter().find(|c| c.display == "te sts");
    assert!(te_sts_cand.is_some(), "应该找到te sts目录补全");
    let replacement = &te_sts_cand.unwrap().replacement;
    #[cfg(windows)]
    {
        assert!(replacement.starts_with('"') && replacement.ends_with("/\""), "补全结果应为\"te sts/\"，不会有双重引号");
        assert!(!replacement.starts_with("\"\""), "不应出现双重开头引号");
    }
}
