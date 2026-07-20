//! Shell 命令执行模块
//! 负责将未知命令转发到系统shell执行，并处理特殊情况如cd命令

use colored::Colorize;
use std::env;
use std::fs;
use std::process::{Command, Stdio};

/// 从 shell 子进程获取最终工作目录的方法
/// 使用临时文件作为跨进程通信媒介，支持所有目录变更操作
fn get_final_shell_pwd(input: &str) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    // 创建临时文件用于传递 pwd
    let temp_pwd_file = env::temp_dir().join(format!("rfe_pwd_{}.tmp", std::process::id()));
    
    #[cfg(windows)]
    let (shell, shell_flag) = {
        if std::env::var("PSModulePath").is_ok() {
            ("powershell.exe", "-Command")
        } else {
            ("cmd", "/c")
        }
    };
    #[cfg(not(windows))]
    let (shell, shell_flag) = ("sh", "-c");
    
    // 构建包装命令：执行用户命令 + 将最终 pwd 写入临时文件
    #[cfg(windows)]
    let wrapped_cmd = if std::env::var("PSModulePath").is_ok() {
        format!(
            "& {{ {}; (Get-Location).Path | Out-File -FilePath '{}' -Encoding UTF8 }}",
            input,
            temp_pwd_file.display()
        )
    } else {
        format!(
            "{} & cd > \"{}\"",
            input,
            temp_pwd_file.display()
        )
    };
    #[cfg(not(windows))]
    let wrapped_cmd = format!("{}; pwd > {}", input, temp_pwd_file.display());
    
    let output = Command::new(shell)
        .arg(shell_flag)
        .arg(&wrapped_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    
    // 读取最终 pwd
    let final_pwd = if temp_pwd_file.exists() {
        match fs::read_to_string(&temp_pwd_file) {
            Ok(content) => {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    Some(trimmed)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };
    
    // 清理临时文件
    let _ = fs::remove_file(&temp_pwd_file);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let mut combined_output = String::new();
    if !stdout.is_empty() {
        combined_output.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !combined_output.is_empty() {
            combined_output.push('\n');
        }
        combined_output.push_str(&stderr);
    }
    
    Ok((combined_output, final_pwd))
}

/// 检测命令是否为简单的cd命令（没有shell操作符）
fn is_cd_command(cmd: &str) -> bool {
    let trimmed = cmd.trim();
    if !trimmed.starts_with("cd ") {
        return false;
    }
    
    // 检查是否包含shell操作符
    let has_shell_ops = trimmed.contains('&') 
        || trimmed.contains('|') 
        || trimmed.contains(';')
        || trimmed.contains('>')
        || trimmed.contains('<')
        || trimmed.contains("&&")
        || trimmed.contains("||");
    
    !has_shell_ops
}



/// 解析cd命令的目标路径
fn parse_cd_target(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();
    if !trimmed.starts_with("cd ") {
        return None;
    }
    
    let path_part = &trimmed[3..];
    let path = path_part.trim();
    
    if path.is_empty() {
        return dirs::home_dir().map(|p| p.to_string_lossy().to_string());
    }
    
    Some(path.to_string())
}

/// 执行cd命令并同步到rfe的工作目录
fn execute_cd(cmd: &str) -> Result<(String, String, Option<String>), Box<dyn std::error::Error>> {
    let target = parse_cd_target(cmd).ok_or("Failed to parse cd target")?;
    
    let current_dir = env::current_dir()?;
    let target_path = shellexpand::tilde(&target).to_string();
    let target_path = std::path::Path::new(&target_path);
    
    if !target_path.exists() {
        return Err(format!("cd: {}: No such file or directory", target).into());
    }
    
    if !target_path.is_dir() {
        return Err(format!("cd: {}: Not a directory", target).into());
    }
    
    env::set_current_dir(target_path)?;
    
    let new_prev_dir = if target_path != current_dir {
        Some(current_dir.display().to_string())
    } else {
        None
    };
    
    let display = format!("Changed to: {}", target_path.display().to_string().cyan());
    let raw = target_path.display().to_string();
    
    Ok((display, raw, new_prev_dir))
}

/// 执行系统shell命令
pub fn cmd_shell(input: &str) -> Result<(String, String, Option<String>), Box<dyn std::error::Error>> {
    let input = input.trim();
    
    if input.is_empty() {
        return Ok((String::new(), String::new(), None));
    }
    
    // 简单 cd 命令使用原有的执行逻辑（有友好的错误提示）
    if is_cd_command(input) {
        return execute_cd(input);
    }
    
    // 使用临时文件跨进程同步工作目录
    // 这种方式支持所有 shell 目录切换操作：cd、cd..、cd.、chdir、pushd、popd、cd - 等
    let (combined_output, final_pwd) = get_final_shell_pwd(input)?;
    
    let display = combined_output.trim_end().to_string();
    
    // 智能提取 raw 输出（用于 {} 占位符替换）：
    // - 仅提取单行内容，不包含任何换行符
    // - 用于命令链之间的数据传递，保证占位符替换的正确性
    let mut raw = String::new();
    let lines: Vec<&str> = display.lines().collect();
    
    if lines.len() == 1 {
        raw = lines[0].trim().to_string();
    } else {
        // 多行时：跳过表头，提取第一个有效非空行
        // 优先提取看起来像路径的行，否则用第一非空行
        let mut first_non_empty: Option<String> = None;
        
        for line in &lines {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                continue;
            }
            
            if first_non_empty.is_none() {
                first_non_empty = Some(trimmed.to_string());
            }
            
            // 跳过明显的表头
            if trimmed.starts_with("----") 
                || trimmed.starts_with("====")
                || trimmed.starts_with("Path")
                || trimmed.starts_with("----") {
                continue;
            }
            
            // 检查是否像路径（包含路径分隔符）
            #[cfg(windows)]
            let looks_like_path = trimmed.contains(':') || trimmed.contains('\\') || trimmed.contains('/');
            #[cfg(not(windows))]
            let looks_like_path = trimmed.starts_with('/') || trimmed.contains('/');
            
            if looks_like_path {
                raw = trimmed.to_string();
                break;
            }
        }
        
        // 如果没找到像路径的行，使用第一非空行
        if raw.is_empty() {
            raw = first_non_empty.unwrap_or_default();
        }
    }
    
    // 确保 raw 输出不包含任何换行符
    raw = raw.replace('\n', "").replace('\r', "");
    
    // 同步 shell 子进程的最终目录到 rfe 主进程
    let new_prev_dir = if let Some(ref final_dir) = final_pwd {
        let current_dir = env::current_dir()?;
        let target_path = std::path::Path::new(final_dir);
        
        if target_path.exists() && target_path.is_dir() && target_path != current_dir {
            env::set_current_dir(target_path)?;
            Some(current_dir.display().to_string())
        } else {
            None
        }
    } else {
        None
    };
    
    Ok((display, raw, new_prev_dir))
}
