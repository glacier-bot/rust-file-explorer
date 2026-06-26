//! 引号处理模块
//! 提供路径引号相关的工具函数

/// 判断路径是否包含需要用双引号包裹的特殊字符
/// 包括：空格、英文括号 () [] {}、& | ; , ^ ! 等会被 shell 或命令解析器拆分的字符
/// 注意：不包含 / \ 这类合法的路径分隔符；也不包含 @ 这类已被解释的前缀
pub fn needs_quoting(path: &str) -> bool {
    path.chars().any(|c| matches!(
        c,
        ' ' | '\t' | '(' | ')' | '[' | ']' | '{' | '}'
            | '&' | '|' | ';' | ',' | '^' | '!' | '`' | '$' | '#'
    ))
}

/// 给补全候选添加双引号包裹（保留尾部斜杠）
/// 例如：`my dir/` -> `"my dir/"`、`my (dir)` -> `"my (dir)"`
/// 如果已经被双引号包裹，则保持不变
pub fn quote_replacement(replacement: &str) -> String {
    if replacement.starts_with('"') && replacement.ends_with('"') && replacement.len() >= 2 {
        return replacement.to_string();
    }
    format!("\"{}\"", replacement)
}
