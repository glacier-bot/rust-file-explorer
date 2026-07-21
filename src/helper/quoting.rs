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

/// 检查路径是否已经被引号包裹（双引号或单引号）
pub fn is_already_quoted(path: &str) -> bool {
    if path.len() < 2 {
        return false;
    }
    let first = path.chars().next().unwrap();
    let last = path.chars().last().unwrap();
    (first == '"' && last == '"') || (first == '\'' && last == '\'')
}

/// 确保路径被双引号包裹（用于运行时执行）
/// 如果路径已经被引号包裹（双引号或单引号），则保持不变
/// 如果路径包含需要引号的特殊字符但没有引号，则添加双引号
/// 如果路径不需要引号，则保持原样
pub fn ensure_quoted(path: &str) -> String {
    if is_already_quoted(path) {
        return path.to_string();
    }
    if needs_quoting(path) {
        format!("\"{}\"", path)
    } else {
        path.to_string()
    }
}


