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
    let mut path: Option<String> = None;
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
            p => path = Some(alias_manager.resolve_path(p)),
        }
        i += 1;
    }

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
    let mut path: Option<String> = None;

    let mut i = arg_offset + 1;
    while i < args.len() {
        match args[i].as_str() {
            "-idx" => {
                is_idx = true;
                if i + 1 < args.len() {
                    idx_tag = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            p => path = Some(alias_manager.resolve_path(p)),
        }
        i += 1;
    }

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
    let mut source: Option<String> = None;
    let mut destination: Option<String> = None;
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
                let resolved = alias_manager.resolve_path(part);
                if source.is_none() {
                    source = Some(resolved);
                } else if destination.is_none() {
                    destination = Some(resolved);
                }
                i += 1;
            }
        }
    }

    let source = source.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
    let destination =
        destination.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;

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
    let path = args
        .get(arg_offset + 1)
        .map(|s| s.as_str())
        .ok_or("Usage: rfe cpf <file>")?;
    Ok(alias_manager.resolve_path(path))
}

/// 解析 open 命令参数
pub fn parse_open_arg(
    args: &[String],
    arg_offset: usize,
    alias_manager: &AliasManager,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = args
        .get(arg_offset + 1)
        .map(|s| s.as_str())
        .ok_or("Usage: rfe open <file>")?;
    Ok(alias_manager.resolve_path(path))
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
