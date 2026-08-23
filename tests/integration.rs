//! 集成测试：通过真实二进制验证四个重构目标的行为
//!
//! 隔离策略：每个用例使用独立的 tempfile 临时目录作为
//! - RFE_CONFIG_DIR（配置/缓存目录，避免污染真实用户配置）
//! - 进程工作目录（mock 文件/文件夹均创建于临时目录，随用例自动清理）

use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

/// 以指定配置目录与工作目录构造一次 rfe 调用
fn rfe(config_dir: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rfe"))
        .args(args)
        .env("RFE_CONFIG_DIR", config_dir)
        .current_dir(cwd)
        .output()
        .expect("failed to spawn rfe binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// 目标1 + 目标2 联合场景：
/// 在 folder1 中以 .index 占位符打标签（不产生实体文件），随后用 cd -tag 跳转；
/// 旧标志 -idx 必须失效
#[test]
fn tag_placeholder_index_and_cd_tag_flag() {
    let config = TempDir::new().unwrap();
    let work = TempDir::new().unwrap();
    let folder1 = work.path().join("folder1");
    std::fs::create_dir(&folder1).unwrap();

    // Given: folder1 下不存在 .index 文件
    // When: 以 .index 占位符添加标签
    let out = rfe(config.path(), &folder1, &["tag", "add", ".index", "newtag1"]);
    assert!(
        out.status.success(),
        "tag add .index should succeed without physical file: {}",
        stderr(&out)
    );

    // Then: .index 仍然不存在（纯占位符）
    assert!(
        !folder1.join(".index").exists(),
        ".index must remain a pure placeholder"
    );

    // Then: tags.json 中的键以 folder1/.index 结尾
    let tags_json = std::fs::read_to_string(config.path().join("tags.json")).unwrap();
    let tags: std::collections::HashMap<String, Vec<String>> =
        serde_json::from_str(&tags_json).unwrap();
    let expected_suffix = format!("folder1{}.index", std::path::MAIN_SEPARATOR);
    assert!(
        tags.keys().any(|key| key.ends_with(&expected_suffix)),
        "tags.json should have a key ending with {}: {:?}",
        expected_suffix,
        tags.keys().collect::<Vec<_>>()
    );

    // Then: tag get 能读取占位符标签
    let out = rfe(config.path(), &folder1, &["tag", "get", ".index"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("newtag1"));

    // When: 使用统一后的 -tag 标志跳转
    let out = rfe(config.path(), work.path(), &["cd", "-tag", "newtag1"]);
    // Then: 成功并打印 folder1 路径
    assert!(
        out.status.success(),
        "cd -tag should succeed: {}",
        stderr(&out)
    );
    assert!(stdout(&out).contains("folder1"));

    // When: 使用已废弃的 -idx 标志
    let out = rfe(config.path(), work.path(), &["cd", "-idx", "newtag1"]);
    // Then: 视为未识别参数而失败
    assert!(!out.status.success(), "cd -idx must be rejected");
}

/// 目标2 边界：真实目录与缺失的普通文件仍然被拒绝
#[test]
fn tag_rejects_directory_and_missing_regular_file() {
    let config = TempDir::new().unwrap();
    let work = TempDir::new().unwrap();
    let folder1 = work.path().join("folder1");
    std::fs::create_dir(&folder1).unwrap();

    // When: 直接给目录打标签
    let out = rfe(
        config.path(),
        work.path(),
        &["tag", "add", folder1.to_str().unwrap(), "t1"],
    );
    // Then: 仍然拒绝（.index 约定不变）
    assert!(!out.status.success());
    assert!(stderr(&out).contains("Cannot add tags to directory"));

    // When: 给不存在的普通文件打标签
    let out = rfe(config.path(), work.path(), &["tag", "add", "missing.txt", "t1"]);
    // Then: 仍然报文件不存在
    assert!(!out.status.success());
    assert!(stderr(&out).contains("File does not exist"));
}

/// 目标2 边界：占位符路径的 ./ 与 重复分隔符 写法归一为同一键
#[test]
fn tag_placeholder_key_is_lexically_normalized() {
    let config = TempDir::new().unwrap();
    let work = TempDir::new().unwrap();
    let folder1 = work.path().join("folder1");
    std::fs::create_dir(&folder1).unwrap();

    // When: 分别以 ".index" 与 "./.index" 两种写法打不同标签
    let out = rfe(config.path(), &folder1, &["tag", "add", ".index", "tag_a"]);
    assert!(out.status.success());
    let out = rfe(
        config.path(),
        &folder1,
        &["tag", "add", "./.index", "tag_b"],
    );
    assert!(out.status.success(), "{}", stderr(&out));

    // Then: 两个标签落在同一个键上
    let out = rfe(config.path(), &folder1, &["tag", "get", ".index"]);
    let text = stdout(&out);
    assert!(text.contains("tag_a"), "{}", text);
    assert!(text.contains("tag_b"), "{}", text);
}

/// 目标3：裸 tag / alias 命令输出末尾显示缓存文件位置
#[test]
fn bare_tag_and_alias_show_cache_file_path() {
    let config = TempDir::new().unwrap();
    let work = TempDir::new().unwrap();

    // When: 运行不带参数的 tag
    let out = rfe(config.path(), work.path(), &["tag"]);
    assert!(out.status.success());
    let text = stdout(&out);
    // Then: 输出包含 tags.json 的完整存放路径
    let expected = config.path().join("tags.json");
    assert!(
        text.contains("tags.json") && text.contains(config.path().to_str().unwrap()),
        "bare tag should print cache path {}: {}",
        expected.display(),
        text
    );

    // When: 运行不带参数的 alias
    let out = rfe(config.path(), work.path(), &["alias"]);
    assert!(out.status.success());
    let text = stdout(&out);
    // Then: 输出包含 aliases.json 的完整存放路径
    assert!(
        text.contains("aliases.json") && text.contains(config.path().to_str().unwrap()),
        "bare alias should print cache path: {}",
        text
    );
}

/// 目标3 边界：list 子命令不追加缓存路径行（仅裸命令有此行为）
#[test]
fn tag_list_subcommand_has_no_cache_path_line() {
    let config = TempDir::new().unwrap();
    let work = TempDir::new().unwrap();

    let out = rfe(config.path(), work.path(), &["tag", "list"]);
    assert!(out.status.success());
    assert!(
        !stdout(&out).contains("Cache file:"),
        "tag list must not append the cache path line"
    );
}
