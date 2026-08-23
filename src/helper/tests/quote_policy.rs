//! 引号策略单元测试：apply_quote_policy 与 quoting 工具函数

use crate::helper::quoting::needs_quoting;

/// 测试引号内补全文件时自动补全结尾引号
#[test]
fn test_apply_quote_policy_in_quote_file() {
    use crate::helper::completion_helpers::apply_quote_policy;
    use rustyline::completion::Pair;

    // 在引号内补全文件（不是目录，没有路径分隔符）
    let mut candidates = vec![
        Pair {
            display: "file.txt".to_string(),
            replacement: "file.txt".to_string(),
        },
        Pair {
            display: "file with spaces.txt".to_string(),
            replacement: "file with spaces.txt".to_string(),
        },
    ];

    apply_quote_policy(&mut candidates, true, '"', false);

    // 文件补全应该自动添加结尾引号
    assert_eq!(candidates[0].replacement, "file.txt\"");
    assert_eq!(candidates[1].replacement, "file with spaces.txt\"");
}

/// 测试引号内补全目录时也添加结尾引号（统一引号策略）
#[test]
fn test_apply_quote_policy_in_quote_directory() {
    use crate::helper::completion_helpers::apply_quote_policy;
    use rustyline::completion::Pair;

    // 在引号内补全目录（以路径分隔符结尾）
    let mut candidates = vec![
        Pair {
            display: "dir/".to_string(),
            replacement: "dir/".to_string(),
        },
        Pair {
            display: "my dir/".to_string(),
            replacement: "my dir/".to_string(),
        },
    ];

    apply_quote_policy(&mut candidates, true, '"', false);

    // 目录补全也添加结尾引号（统一引号策略）
    assert_eq!(candidates[0].replacement, "dir/\"");
    assert_eq!(candidates[1].replacement, "my dir/\"");
}

/// 测试单引号内补全文件时自动补全结尾引号
#[test]
fn test_apply_quote_policy_in_single_quote() {
    use crate::helper::completion_helpers::apply_quote_policy;
    use rustyline::completion::Pair;

    // 在单引号内补全文件
    let mut candidates = vec![Pair {
        display: "file.txt".to_string(),
        replacement: "file.txt".to_string(),
    }];

    apply_quote_policy(&mut candidates, true, '\'', false);

    // 文件补全应该自动添加结尾单引号
    assert_eq!(candidates[0].replacement, "file.txt'");
}

/// 测试引号内补全不会出现多余引号
#[test]
fn test_apply_quote_policy_no_extra_quotes() {
    use crate::helper::completion_helpers::apply_quote_policy;
    use rustyline::completion::Pair;

    // 场景1：补全结果本身已经带引号
    let mut candidates1 = vec![Pair {
        display: "file.txt".to_string(),
        replacement: "\"file.txt\"".to_string(),
    }];
    apply_quote_policy(&mut candidates1, true, '"', false);
    assert_eq!(candidates1[0].replacement, "file.txt\""); // 不应有开头引号，只应有结尾引号

    // 场景2：补全结果只带开头引号（FilenameCompleter 的常见行为）
    let mut candidates2 = vec![Pair {
        display: "file with space.txt".to_string(),
        replacement: "\"file with space.txt".to_string(),
    }];
    apply_quote_policy(&mut candidates2, true, '"', false);
    assert_eq!(candidates2[0].replacement, "file with space.txt\"");

    // 场景3：补全结果带单引号，但用户用的是双引号
    let mut candidates3 = vec![Pair {
        display: "file.txt".to_string(),
        replacement: "'file.txt'".to_string(),
    }];
    apply_quote_policy(&mut candidates3, true, '"', false);
    assert_eq!(candidates3[0].replacement, "file.txt\"");

    // 场景4：单引号内补全，结果带双引号
    let mut candidates4 = vec![Pair {
        display: "file.txt".to_string(),
        replacement: "\"file.txt\"".to_string(),
    }];
    apply_quote_policy(&mut candidates4, true, '\'', false);
    assert_eq!(candidates4[0].replacement, "file.txt'");

    // 场景5：路径中间有引号字符（虽然实际路径不应该有，但要处理）
    let mut candidates5 = vec![Pair {
        display: "file\"name.txt".to_string(),
        replacement: "file\"name.txt".to_string(),
    }];
    apply_quote_policy(&mut candidates5, true, '"', false);
    assert_eq!(candidates5[0].replacement, "filename.txt\""); // 中间引号应该被移除

    // 场景6：目录补全，结果带引号
    let mut candidates6 = vec![Pair {
        display: "my dir/".to_string(),
        replacement: "\"my dir/".to_string(),
    }];
    apply_quote_policy(&mut candidates6, true, '"', false);
    assert_eq!(candidates6[0].replacement, "my dir/\""); // 斜杠后只有一个结尾引号

    // 场景7：用户输入路径中间包含反斜杠（Windows 路径）
    // 输入: cd "te sts\in，光标在n后
    // FilenameCompleter 可能返回 "in dex.txt" 或其他带引号形式
    let mut candidates7 = vec![Pair {
        display: "in dex.txt".to_string(),
        replacement: "\"in dex.txt\"".to_string(), // 补全结果自带完整引号
    }];
    apply_quote_policy(&mut candidates7, true, '"', false);
    assert_eq!(candidates7[0].replacement, "in dex.txt\""); // 只应该有一个结尾引号

    // 场景8：补全结果只带开头引号（Windows FilenameCompleter 常见行为）
    let mut candidates8 = vec![Pair {
        display: "in dex.txt".to_string(),
        replacement: "\"in dex.txt".to_string(), // 只有开头引号
    }];
    apply_quote_policy(&mut candidates8, true, '"', false);
    assert_eq!(candidates8[0].replacement, "in dex.txt\"");
}

/// 测试 needs_quoting 辅助函数对各类特殊字符的识别
#[test]
fn test_needs_quoting_special_chars() {
    // 不含特殊字符
    assert!(!needs_quoting("simple"));
    assert!(!needs_quoting("path/to/file.txt"));
    assert!(!needs_quoting("C:\\Users\\q\\Desktop"));
    assert!(!needs_quoting("中文路径"));

    // 含空格
    assert!(needs_quoting("my folder"));
    assert!(needs_quoting("a b"));

    // 含英文括号
    assert!(needs_quoting("Program Files (x86)"));
    assert!(needs_quoting("dir(1)"));
    assert!(needs_quoting("[bracket]"));
    assert!(needs_quoting("{brace}"));

    // 其他 shell 特殊字符
    assert!(needs_quoting("a&b"));
    assert!(needs_quoting("a|b"));
    assert!(needs_quoting("a;b"));
    assert!(needs_quoting("a,b"));
}

/// 测试 is_already_quoted 辅助函数
#[test]
fn test_is_already_quoted() {
    use crate::helper::quoting::is_already_quoted;

    // 空字符串和单个字符
    assert!(!is_already_quoted(""));
    assert!(!is_already_quoted("\""));
    assert!(!is_already_quoted("'"));

    // 双引号包裹
    assert!(is_already_quoted(r#""path""#));
    assert!(is_already_quoted(r#""my path""#));

    // 单引号包裹
    assert!(is_already_quoted("'path'"));
    assert!(is_already_quoted("'my path'"));

    // 不匹配的引号
    assert!(!is_already_quoted(r#""path'#));
    assert!(!is_already_quoted("'path\""));

    // 只有开头引号
    assert!(!is_already_quoted(r#""path"#));
    assert!(!is_already_quoted("'path"));

    // 只有结尾引号
    assert!(!is_already_quoted(r#"path""#));
    assert!(!is_already_quoted("path'"));

    // 没有引号
    assert!(!is_already_quoted("path"));
    assert!(!is_already_quoted("my path"));
}

/// 测试 ensure_quoted 辅助函数
#[test]
fn test_ensure_quoted() {
    use crate::helper::quoting::ensure_quoted;

    // 已被双引号包裹则保持不变
    assert_eq!(ensure_quoted(r#""my path""#), r#""my path""#);

    // 已被单引号包裹则保持不变
    assert_eq!(ensure_quoted("'my path'"), "'my path'");

    // 无论是否包含空格，始终添加双引号
    assert_eq!(ensure_quoted("my path"), r#""my path""#);
    assert_eq!(ensure_quoted("Program Files"), r#""Program Files""#);
    assert_eq!(ensure_quoted("simple"), r#""simple""#);
    assert_eq!(ensure_quoted("path/to/file.txt"), r#""path/to/file.txt""#);
    assert_eq!(ensure_quoted("C:\\Users\\q\\Desktop"), r#""C:\Users\q\Desktop""#);
    assert_eq!(ensure_quoted("中文路径"), r#""中文路径""#);

    // 包含括号和其他特殊字符也添加双引号
    assert_eq!(ensure_quoted("dir(1)"), r#""dir(1)""#);
    assert_eq!(ensure_quoted("Program Files (x86)"), r#""Program Files (x86)""#);
    assert_eq!(ensure_quoted("a&b"), r#""a&b""#);
    assert_eq!(ensure_quoted("file;name"), r#""file;name""#);

    // 保留尾部斜杠
    assert_eq!(ensure_quoted("my dir/"), r#""my dir/""#);
    assert_eq!(ensure_quoted("dir(1)/"), r#""dir(1)/""#);
}
