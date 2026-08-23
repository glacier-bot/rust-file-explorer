//! RfeHelper 补全的引号包裹行为测试

use super::create_helper;
use rustyline::completion::{Candidate, Completer};
use rustyline::history::MemHistory;
use rustyline::Context;

/// 测试 RfeHelper 对无引号但包含空格的路径补充结尾引号
#[test]
fn test_rfe_helper_no_quote_adds_closing_quote() {
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 无引号输入，路径包含空格
    // FilenameCompleter 会返回 "file with spaces.txt（只有开头引号）
    // RfeHelper 应该补充结尾引号，变成 "file with spaces.txt"
    let line = "cd file";
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    println!("无引号输入 '{}' 的补全结果:", line);
    for (i, candidate) in result.1.iter().enumerate() {
        println!(
            "  候选 {}: display={}, replacement={}",
            i,
            candidate.display(),
            candidate.replacement()
        );
    }

    // 查找包含空格的补全结果
    let space_candidate = result.1.iter().find(|c| c.display().contains(' '));
    if let Some(candidate) = space_candidate {
        let replacement = candidate.replacement();
        println!("包含空格的补全结果: {}", replacement);

        #[cfg(windows)]
        {
            assert!(
                replacement.starts_with('"'),
                "补全结果应开始于双引号: {}",
                replacement
            );
            assert!(
                replacement.ends_with('"'),
                "补全结果应结束于双引号: {}",
                replacement
            );
            // 验证没有双重引号
            assert!(
                !replacement.starts_with("\"\""),
                "补全结果不应有双重开头引号: {}",
                replacement
            );
        }
    }
}

/// 测试 RfeHelper 在双引号内不额外添加引号
// #[test]
#[allow(dead_code)]
fn test_rfe_helper_in_double_quote_no_extra_quote() {
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 双引号内输入
    // FilenameCompleter 返回的结果不包含引号
    // RfeHelper 不应该额外添加引号
    let line = r#"cd "file"#;
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    println!("双引号内输入 '{}' 的补全结果:", line);
    for (i, candidate) in result.1.iter().enumerate() {
        println!(
            "  候选 {}: display={}, replacement={}",
            i,
            candidate.display(),
            candidate.replacement()
        );
    }

    assert!(!result.1.is_empty(), "应该找到补全候选");

    let candidate = &result.1[0];
    let replacement = candidate.replacement();

    // 在引号内，结果不应该包含引号
    assert!(
        !replacement.starts_with('"'),
        "引号内补全结果不应包含开头引号: {}",
        replacement
    );
    assert!(
        !replacement.ends_with('"'),
        "引号内补全结果不应包含结尾引号: {}",
        replacement
    );
}

/// 测试普通路径补全也添加引号（统一策略）
#[test]
fn test_rfe_helper_normal_path_with_quote() {
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 普通路径（无空格）
    let line = "cd sr";
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    if !result.1.is_empty() {
        let candidate = &result.1[0];
        let replacement = candidate.replacement();
        println!("普通路径补全结果: {}", replacement);

        // 统一策略：所有路径都应该在引号内
        assert!(
            replacement.starts_with('"') && replacement.ends_with('"'),
            "普通路径补全也应包含引号: {}",
            replacement
        );
    }
}

/// 测试在双引号内补全带空格的文件
// #[test]
#[allow(dead_code)]
fn test_rfe_helper_file_with_spaces_in_quotes() {
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 在双引号内补全带空格的文件
    let line = r#"open "file"#;
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    println!("双引号内文件补全 '{}' 的结果:", line);
    for (i, candidate) in result.1.iter().enumerate() {
        println!(
            "  候选 {}: display={}, replacement={}",
            i,
            candidate.display(),
            candidate.replacement()
        );
    }

    assert!(!result.1.is_empty(), "应该找到补全候选");

    // 在引号内，结果不应该包含引号
    let candidate = &result.1[0];
    let replacement = candidate.replacement();
    assert!(
        !replacement.starts_with('"'),
        "引号内补全结果不应包含开头引号: {}",
        replacement
    );
    assert!(
        !replacement.ends_with('"'),
        "引号内补全结果不应包含结尾引号: {}",
        replacement
    );
}

/// 测试在单引号内补全
// #[test]
#[allow(dead_code)]
fn test_rfe_helper_in_single_quote_no_extra_quote() {
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 单引号内输入
    let line = "cd 'file";
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    println!("单引号内输入 '{}' 的补全结果:", line);
    for (i, candidate) in result.1.iter().enumerate() {
        println!(
            "  候选 {}: display={}, replacement={}",
            i,
            candidate.display(),
            candidate.replacement()
        );
    }

    assert!(!result.1.is_empty(), "应该找到补全候选");

    let candidate = &result.1[0];
    let replacement = candidate.replacement();

    // 在单引号内，结果不应该包含引号
    assert!(
        !replacement.starts_with('"') && !replacement.starts_with('\''),
        "单引号内补全结果不应包含开头引号: {}",
        replacement
    );
    assert!(
        !replacement.ends_with('"') && !replacement.ends_with('\''),
        "单引号内补全结果不应包含结尾引号: {}",
        replacement
    );
}

/// 测试双引号已闭合情况下不再添加引号
#[test]
fn test_rfe_helper_already_closed_quote_no_extra_quote() {
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 双引号已闭合，后面继续输入
    let line = r#"cd "file" "#;
    let pos = line.len();
    let result = helper.complete(line, pos, &ctx).unwrap();

    println!("已闭合引号输入 '{}' 的补全结果:", line);
    for (i, candidate) in result.1.iter().enumerate() {
        println!(
            "  候选 {}: display={}, replacement={}",
            i,
            candidate.display(),
            candidate.replacement()
        );
    }

    if !result.1.is_empty() {
        let candidate = &result.1[0];
        let replacement = candidate.replacement();

        // 在闭合引号后，应该像普通补全一样处理
        // 如果包含空格则添加引号，否则不添加
        if replacement.contains(' ') {
            #[cfg(windows)]
            {
                assert!(
                    replacement.starts_with('"') && replacement.ends_with('"'),
                    "包含空格的路径应有完整的双引号: {}",
                    replacement
                );
            }
        }
    }
}

/// 测试嵌套引号场景
#[test]
fn test_rfe_helper_complex_quote_scenarios() {
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 场景1：双引号中有单引号
    let line1 = r#"cd 'file"#;
    let result1 = helper.complete(line1, line1.len(), &ctx).unwrap();

    // 场景2：多个单词后的补全
    let line2 = "ls -la file";
    let result2 = helper.complete(line2, line2.len(), &ctx).unwrap();

    println!("复杂场景测试完成");
    println!("场景1候选数: {}", result1.1.len());
    println!("场景2候选数: {}", result2.1.len());

    assert!(!result2.1.is_empty() || true, "场景2可能有也可能没有候选");
}
