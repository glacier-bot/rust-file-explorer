//! 路径补全测试：@alias 子路径、cd -r 行号路径与默认文件名补全

use super::create_helper;
use rustyline::completion::Completer;
use rustyline::history::MemHistory;
use rustyline::Context;

/// 测试 @alias 子路径补全在含特殊字符路径下统一加引号
/// 通过模拟一个含括号的别名目录验证
#[test]
fn test_alias_sub_path_completion_with_special_chars() {
    use std::fs;
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 创建临时目录结构：tmp_root/sub (1)/inner.txt
    let tmp_root = std::env::temp_dir().join("rfe_test_alias_special");
    let _ = fs::remove_dir_all(&tmp_root);
    let sub_dir = tmp_root.join("sub (1)");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("inner.txt"), "x").unwrap();

    // 注册别名指向 tmp_root（直接操作 HashMap 避免污染真实配置）
    {
        let mut mgr = helper.alias_manager.lock().unwrap();
        mgr.aliases.insert(
            "rfe_special_alias".to_string(),
            tmp_root.to_string_lossy().to_string(),
        );
    }

    // 触发 @alias/ 子路径补全
    let line = "cd @rfe_special_alias/";
    let result = helper.complete(line, line.len(), &ctx).unwrap();

    let dir_candidate = result
        .1
        .iter()
        .find(|c| c.display.contains("sub (1)"));
    assert!(dir_candidate.is_some(), "应包含含括号的目录候选");
    let replacement = &dir_candidate.unwrap().replacement;
    assert!(
        replacement.starts_with('"') && replacement.ends_with('"'),
        "含括号的别名子路径补全应被双引号包裹: {}",
        replacement
    );

    // 清理
    let _ = fs::remove_dir_all(&tmp_root);
    let mut mgr = helper.alias_manager.lock().unwrap();
    mgr.aliases.remove("rfe_special_alias");
}

/// 测试默认文件名补全对仅含括号（无空格）特殊字符的路径也加双引号
#[test]
fn test_default_completion_quotes_parentheses() {
    use std::fs;
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    let tmp_root = std::env::temp_dir().join("rfe_test_paren");
    let _ = fs::remove_dir_all(&tmp_root);
    fs::create_dir_all(&tmp_root).unwrap();
    fs::create_dir_all(tmp_root.join("paren(only)")).unwrap();

    let prefix = tmp_root.join("paren").to_string_lossy().to_string();
    let line = format!("cd {}", prefix);
    let result = helper.complete(&line, line.len(), &ctx).unwrap();

    let cand = result
        .1
        .iter()
        .find(|c| c.replacement.contains("paren(only)"));
    if let Some(c) = cand {
        assert!(
            c.replacement.starts_with('"') && c.replacement.trim_end_matches('/').ends_with('"')
                || c.replacement.ends_with('"'),
            "含括号的补全候选应被双引号包裹: {}",
            c.replacement
        );
    }

    let _ = fs::remove_dir_all(&tmp_root);
}

/// 测试 cd -r 使用正斜杠和反斜杠的补全功能
#[test]
fn test_cd_r_with_both_separators() {
    use std::fs;
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 创建临时目录结构
    let tmp_root = std::env::temp_dir().join("rfe_test_cd_r_sep");
    let _ = fs::remove_dir_all(&tmp_root);
    fs::create_dir_all(&tmp_root).unwrap();
    let sub_dir = tmp_root.join("test_subdir");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("inner_file.txt"), "test").unwrap();

    // 填充 last_ls_items
    {
        let mut items = helper.last_ls_items.lock().unwrap();
        items.push(crate::models::FileInfo {
            name: "tmp_root".to_string(),
            full_path: tmp_root.to_string_lossy().to_string(),
            icon: "📁",
            color: colored::Color::Blue,
            size: 0,
            created: None,
            modified: std::time::SystemTime::now(),
            is_dir: true,
            tags: vec![],
        });
    }

    // 测试正斜杠: cd -r 1/
    let line_fwd = "cd -r 1/";
    let result_fwd = helper.complete(line_fwd, line_fwd.len(), &ctx).unwrap();
    assert!(!result_fwd.1.is_empty(), "使用正斜杠应该有补全结果");

    // 检查补全是否使用正斜杠
    let subdir_cand_fwd = result_fwd.1.iter().find(|c| c.display == "test_subdir");
    assert!(subdir_cand_fwd.is_some(), "应该找到test_subdir");
    assert!(subdir_cand_fwd.unwrap().replacement.contains("1/"), "补全结果应使用正斜杠");

    // 测试反斜杠: cd -r 1\
    let line_back = r"cd -r 1\";
    let result_back = helper.complete(line_back, line_back.len(), &ctx).unwrap();
    assert!(!result_back.1.is_empty(), "使用反斜杠应该有补全结果");

    // 检查补全是否使用反斜杠
    let subdir_cand_back = result_back.1.iter().find(|c| c.display == "test_subdir");
    assert!(subdir_cand_back.is_some(), "应该找到test_subdir");
    assert!(subdir_cand_back.unwrap().replacement.contains(r"1\"), "补全结果应使用反斜杠");

    // 清理
    let _ = fs::remove_dir_all(&tmp_root);
}

/// 测试 @alias 子路径补全使用正斜杠和反斜杠
#[test]
fn test_alias_completion_with_both_separators() {
    use std::fs;
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 创建临时目录结构
    let tmp_root = std::env::temp_dir().join("rfe_test_alias_sep");
    let _ = fs::remove_dir_all(&tmp_root);
    fs::create_dir_all(&tmp_root).unwrap();
    let sub_dir = tmp_root.join("alias_subdir");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(sub_dir.join("alias_file.txt"), "test").unwrap();

    // 注册别名
    {
        let mut mgr = helper.alias_manager.lock().unwrap();
        mgr.aliases.insert(
            "test_alias".to_string(),
            tmp_root.to_string_lossy().to_string(),
        );
    }

    // 测试正斜杠: cd @test_alias/
    let line_fwd = "cd @test_alias/";
    let result_fwd = helper.complete(line_fwd, line_fwd.len(), &ctx).unwrap();
    assert!(!result_fwd.1.is_empty(), "使用正斜杠的别名补全应该有结果");

    let subdir_cand_fwd = result_fwd.1.iter().find(|c| c.display == "alias_subdir");
    assert!(subdir_cand_fwd.is_some(), "应该找到alias_subdir");
    assert!(subdir_cand_fwd.unwrap().replacement.contains("@test_alias/"), "应使用正斜杠");

    // 测试反斜杠: cd @test_alias\
    let line_back = r"cd @test_alias\";
    let result_back = helper.complete(line_back, line_back.len(), &ctx).unwrap();
    assert!(!result_back.1.is_empty(), "使用反斜杠的别名补全应该有结果");

    let subdir_cand_back = result_back.1.iter().find(|c| c.display == "alias_subdir");
    assert!(subdir_cand_back.is_some(), "应该找到alias_subdir");
    assert!(subdir_cand_back.unwrap().replacement.contains(r"@test_alias\"), "应使用反斜杠");

    // 清理
    let _ = fs::remove_dir_all(&tmp_root);
    let mut mgr = helper.alias_manager.lock().unwrap();
    mgr.aliases.remove("test_alias");
}

/// 测试 cd -r 子路径补全的深层目录
#[test]
fn test_cd_r_deep_subpath_completion() {
    use std::fs;
    let helper = create_helper();
    let history = MemHistory::default();
    let ctx = Context::new(&history);

    // 创建深层目录结构
    let tmp_root = std::env::temp_dir().join("rfe_test_cd_r_deep");
    let _ = fs::remove_dir_all(&tmp_root);
    let deep_dir = tmp_root.join("level1").join("level2").join("level3");
    fs::create_dir_all(&deep_dir).unwrap();
    fs::write(deep_dir.join("deep_file.txt"), "test").unwrap();

    // 填充 last_ls_items
    {
        let mut items = helper.last_ls_items.lock().unwrap();
        items.push(crate::models::FileInfo {
            name: "deep_root".to_string(),
            full_path: tmp_root.to_string_lossy().to_string(),
            icon: "📁",
            color: colored::Color::Blue,
            size: 0,
            created: None,
            modified: std::time::SystemTime::now(),
            is_dir: true,
            tags: vec![],
        });
    }

    // 测试深层路径，混合使用分隔符也能处理
    let line = "cd -r 1/level1/level2/";
    let result = helper.complete(line, line.len(), &ctx).unwrap();
    assert!(!result.1.is_empty(), "深层路径补全应该有结果");

    let level3_cand = result.1.iter().find(|c| c.display == "level3");
    assert!(level3_cand.is_some(), "应该找到level3目录");

    // 清理
    let _ = fs::remove_dir_all(&tmp_root);
}
