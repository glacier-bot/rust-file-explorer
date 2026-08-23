use std::fs;
use std::path::PathBuf;

pub fn is_hidden(path: &PathBuf) -> bool {
    #[cfg(unix)]
    {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        matches!(fs::metadata(path), Ok(meta) if (meta.file_attributes() & 2) != 0)
    }
}

#[derive(Debug, PartialEq)]
pub struct PopResult {
    pub path: String,
    pub actual_pops: usize,
    pub reached_boundary: bool,
}

pub fn pop_path(path: &str, pop_count: usize) -> PopResult {
    if path.is_empty() {
        return PopResult {
            path: String::new(),
            actual_pops: 0,
            reached_boundary: pop_count > 0,
        };
    }

    let mut path_buf = PathBuf::from(path);
    let mut actual_pops = 0;
    let mut reached_boundary = false;

    for _ in 0..pop_count {
        let current_path = path_buf.clone();
        match path_buf.parent() {
            Some(parent) => {
                let parent_str = parent.to_string_lossy();
                let current_str = current_path.to_string_lossy();
                
                if parent_str == current_str {
                    reached_boundary = true;
                    break;
                }
                
                path_buf = parent.to_path_buf();
                actual_pops += 1;
            }
            None => {
                reached_boundary = true;
                break;
            }
        }
    }

    PopResult {
        path: path_buf.to_string_lossy().to_string(),
        actual_pops,
        reached_boundary,
    }
}

#[derive(Debug, PartialEq)]
pub struct ExpandResult {
    pub expanded: String,
    pub reached_boundary: bool,
    pub actual_pops: usize,
    pub total_replacements: usize,
}

pub fn expand_pop_placeholders(cmd: &str, previous_raw_data: &str) -> ExpandResult {
    let mut result = String::with_capacity(cmd.len() + previous_raw_data.len());
    let chars: Vec<char> = cmd.chars().collect();
    let mut i = 0;
    let mut reached_boundary = false;
    let mut last_actual_pops = 0;
    let mut total_replacements = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '}' {
            let mut j = i + 2;
            let mut pop_count = 0;

            while j < chars.len() && chars[j] == '.' {
                if j + 3 < chars.len()
                    && chars[j + 1] == 'p'
                    && chars[j + 2] == 'o'
                    && chars[j + 3] == 'p'
                {
                    pop_count += 1;
                    j += 4;
                } else {
                    pop_count += 1;
                    j += 1;
                }
            }

            let pop_result = pop_path(previous_raw_data, pop_count);
            reached_boundary |= pop_result.reached_boundary;
            last_actual_pops = pop_result.actual_pops;
            result.push_str(&pop_result.path);
            total_replacements += 1;
            i = j;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    ExpandResult {
        expanded: result,
        reached_boundary,
        actual_pops: last_actual_pops,
        total_replacements,
    }
}

#[cfg(test)]
mod tests;
