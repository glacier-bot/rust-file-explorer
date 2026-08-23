//! 引号上下文判断与引号策略模块
//! 提供补全过程中的引号状态检测与统一引号处理

use rustyline::completion::Pair;

use super::quoting::ensure_quoted;

/// 检查当前输入是否处于引号内，并返回引号类型
pub fn check_in_quote(line: &str, pos: usize) -> (bool, char) {
    let mut in_quote = false;
    let mut quote_char = '"';
    for c in line[..pos].chars() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
            }
            '"' | '\'' if in_quote && c == quote_char => {
                in_quote = false;
            }
            _ => {}
        }
    }
    (in_quote, quote_char)
}

/// 检查光标是否在闭合引号之后
pub fn is_after_closing_quote(line: &str, pos: usize) -> bool {
    if pos > 0 {
        let last_char = line[..pos].chars().last().unwrap();
        last_char == '"' || last_char == '\''
    } else {
        false
    }
}

/// 检查光标是否在闭合引号之前（即光标后面紧跟一个闭合引号）
pub fn is_before_closing_quote(line: &str, pos: usize) -> bool {
    if pos < line.len() {
        let next_char = line[pos..].chars().next().unwrap();
        next_char == '"' || next_char == '\''
    } else {
        false
    }
}

/// 对补全结果应用统一的引号策略
/// 原则：引号必须只闭合1次，无论路径是否有空格都放到引号内
pub fn apply_quote_policy(
    candidates: &mut Vec<Pair>,
    in_quote: bool,
    quote_char: char,
    cursor_before_closing_quote: bool,  // 新增：光标是否在闭合引号之前
) {
    if in_quote {
        // 在引号内：移除补全结果中的所有引号
        for candidate in candidates {
            let repl = &candidate.replacement;

            // 移除所有开头和结尾的引号
            let mut base_repl = repl.trim_matches(|c| c == '"' || c == '\'').to_string();

            // 移除内容中多余的引号字符
            base_repl = base_repl.replace(quote_char, "");

            // 只有当光标不在闭合引号之前时，才添加结尾引号
            // 如果光标已经在闭合引号之前了，用户已经输入了结尾引号，不需要再加
            if !cursor_before_closing_quote {
                base_repl.push(quote_char);
            }

            candidate.replacement = base_repl;
        }
    } else {
        // 不在引号内：确保结果被一对引号包裹（只有开头和结尾各一个）
        for candidate in candidates {
            let repl = &candidate.replacement;

            // 移除所有引号，然后统一添加一对
            let trimmed = repl.replace('"', "").replace('\'', "");

            // 使用 ensure_quoted 统一添加一对引号
            candidate.replacement = ensure_quoted(&trimmed);
        }
    }
}
