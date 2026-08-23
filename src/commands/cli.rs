//! 命令行参数解析模块
//! 提供统一的参数解析接口，让每个命令自己处理参数解析

use crate::managers::alias::AliasManager;
use regex::Regex;

/// ls 命令参数
pub struct LsArgs {
    pub all: bool,
    pub long: bool,
    pub re: bool,
    pub re_insensitive: bool,
    pub show_tags: bool,
    pub recursive: bool,
    pub path: Option<String>,
    pub tag_patterns: Vec<Regex>,
}

/// cd 命令参数
pub struct CdArgs {
    pub path: Option<String>,
    pub is_idx: bool,
    pub idx_tag: Option<String>,
}

/// mv 命令参数
pub struct MvArgs {
    pub source: String,
    pub destination: String,
    pub copy: bool,
}

/// 解析 ls 命令参数
pub fn parse_ls_args(
    args: &[String],
    arg_offset: usize,
    alias_manager: &AliasManager,
) -> Result<LsArgs, Box<dyn std::error::Error>> {
    let mut all = false;
    let mut long = false;
    let mut re = false;
    let mut re_insensitive = false;
    let mut show_tags = false;
    let mut recursive = false;
    let mut path_parts: Vec<String> = Vec::new();
    let mut tag_pattern_strs: Vec<String> = Vec::new();

    let mut i = arg_offset + 1;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => all = true,
            "-l" | "--long" => long = true,
            "-la" | "-al" => {
                all = true;
                long = true;
            }
            "--re" => re = true,
            "--re-deep" => {
                re = true;
                recursive = true;
            }
            "--re-insensitive" => re_insensitive = true,
            "--xcaps" => re_insensitive = true,
            "-tag" | "--tags" => show_tags = true,
            "-t" | "--tag" => {
                if i + 1 < args.len() {
                    if args[i + 1] == "--deep" && i + 2 < args.len() {
                        recursive = true;
                        tag_pattern_strs.push(args[i + 2].clone());
                        i += 2;
                    } else {
                        tag_pattern_strs.push(args[i + 1].clone());
                        i += 1;
                    }
                } else {
                    return Err(
                        "Tag query parameter requires a pattern, usage: ls -t <tag_regex>".into(),
                    );
                }
            }
            p => path_parts.push(alias_manager.resolve_path(p)),
        }
        i += 1;
    }

    let path = if path_parts.is_empty() {
        None
    } else {
        Some(path_parts.join(" "))
    };

    let mut tag_patterns = Vec::new();
    for pattern_str in tag_pattern_strs {
        match Regex::new(&pattern_str) {
            Ok(re) => tag_patterns.push(re),
            Err(e) => return Err(format!("Invalid tag regex: {}", e).into()),
        }
    }

    Ok(LsArgs {
        all,
        long,
        re,
        re_insensitive,
        show_tags,
        recursive,
        path,
        tag_patterns,
    })
}

/// 解析 cd 命令参数
pub fn parse_cd_args(
    args: &[String],
    arg_offset: usize,
    alias_manager: &AliasManager,
) -> Result<CdArgs, Box<dyn std::error::Error>> {
    let mut is_idx = false;
    let mut idx_tag: Option<String> = None;
    let mut path_parts: Vec<String> = Vec::new();

    let mut i = arg_offset + 1;
    while i < args.len() {
        match args[i].as_str() {
            "-tag" => {
                is_idx = true;
                if i + 1 < args.len() {
                    idx_tag = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            p => path_parts.push(alias_manager.resolve_path(p)),
        }
        i += 1;
    }

    let path = if path_parts.is_empty() {
        None
    } else {
        Some(path_parts.join(" "))
    };

    Ok(CdArgs {
        path,
        is_idx,
        idx_tag,
    })
}

/// 解析 mv 命令参数
pub fn parse_mv_args(
    args: &[String],
    arg_offset: usize,
    alias_manager: &AliasManager,
) -> Result<MvArgs, Box<dyn std::error::Error>> {
    let mut path_parts: Vec<String> = Vec::new();
    let mut copy = false;

    let mut i = arg_offset + 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cp" => {
                copy = true;
                i += 1;
            }
            "-r" => {
                return Err("-r parameter is only available in REPL mode (interactive mode)".into());
            }
            part => {
                path_parts.push(alias_manager.resolve_path(part));
                i += 1;
            }
        }
    }

    if path_parts.len() < 2 {
        return Err("Usage: rfe mv <source_path> <destination_path> [--cp]".into());
    }

    let destination = path_parts.pop().unwrap();
    let source = path_parts.join(" ");

    Ok(MvArgs {
        source,
        destination,
        copy,
    })
}

/// 解析 cpf 命令参数
pub fn parse_cpf_arg(
    args: &[String],
    arg_offset: usize,
    alias_manager: &AliasManager,
) -> Result<String, Box<dyn std::error::Error>> {
    let path_parts: Vec<String> = args[arg_offset + 1..]
        .iter()
        .map(|s| alias_manager.resolve_path(s))
        .collect();
    
    if path_parts.is_empty() {
        return Err("Usage: rfe cpf <file>".into());
    }
    
    Ok(path_parts.join(" "))
}

/// 解析 open 命令参数
pub fn parse_open_arg(
    args: &[String],
    arg_offset: usize,
    alias_manager: &AliasManager,
) -> Result<String, Box<dyn std::error::Error>> {
    let path_parts: Vec<String> = args[arg_offset + 1..]
        .iter()
        .map(|s| alias_manager.resolve_path(s))
        .collect();
    
    if path_parts.is_empty() {
        return Err("Usage: rfe open <file>".into());
    }
    
    Ok(path_parts.join(" "))
}

/// 解析 alias 命令参数（直接返回切片）
pub fn get_alias_args(args: &[String], arg_offset: usize) -> Vec<&str> {
    args[arg_offset + 1..].iter().map(|s| s.as_str()).collect()
}

/// 解析 tag 命令参数（直接返回切片）
pub fn get_tag_args(args: &[String], arg_offset: usize) -> Vec<&str> {
    args[arg_offset + 1..].iter().map(|s| s.as_str()).collect()
}

/// 解析 mkdf 命令参数（直接返回切片）
pub fn get_mkdf_args(args: &[String], arg_offset: usize) -> Vec<&str> {
    args[arg_offset + 1..].iter().map(|s| s.as_str()).collect()
}

/// 解析 change 命令参数（直接返回切片）
pub fn get_change_args(args: &[String], arg_offset: usize) -> Vec<&str> {
    args[arg_offset + 1..].iter().map(|s| s.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    /// 构造配置隔离的 AliasManager 及其临时目录（目录随测试结束自动清理）
    fn test_alias_manager() -> (TempDir, AliasManager) {
        let dir = tempdir().unwrap();
        let manager = AliasManager::with_config_dir(dir.path().to_path_buf()).unwrap();
        (dir, manager)
    }

    #[test]
    fn test_parse_cd_args_accepts_unified_tag_flag() {
        let (_dir, manager) = test_alias_manager();
        let args: Vec<String> = ["rfe", "cd", "-tag", "work"].iter().map(|s| s.to_string()).collect();
        let parsed = parse_cd_args(&args, 1, &manager).unwrap();
        assert!(parsed.is_idx);
        assert_eq!(parsed.idx_tag.as_deref(), Some("work"));
        assert!(parsed.path.is_none());
    }

    #[test]
    fn test_parse_cd_args_removed_idx_flag_is_treated_as_path() {
        let (_dir, manager) = test_alias_manager();
        let args: Vec<String> = ["rfe", "cd", "-idx", "work"].iter().map(|s| s.to_string()).collect();
        let parsed = parse_cd_args(&args, 1, &manager).unwrap();
        // -idx 已统一移除：按普通路径参数处理，不再触发按标签跳转
        assert!(!parsed.is_idx);
        assert_eq!(parsed.idx_tag, None);
        assert_eq!(parsed.path.as_deref(), Some("-idx work"));
    }

    #[test]
    fn test_parse_cd_args_plain_path() {
        let (_dir, manager) = test_alias_manager();
        let args: Vec<String> = ["rfe", "cd", "src"].iter().map(|s| s.to_string()).collect();
        let parsed = parse_cd_args(&args, 1, &manager).unwrap();
        assert!(!parsed.is_idx);
        assert_eq!(parsed.idx_tag, None);
        assert_eq!(parsed.path.as_deref(), Some("src"));
    }
}
