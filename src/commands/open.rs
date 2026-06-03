use colored::*;
use std::path::PathBuf;
use std::process::Command;

pub fn cmd_open(path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let target = PathBuf::from(path);

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    let plain_path = target.display().to_string();

    if target.is_dir() {
        #[cfg(target_os = "windows")]
        Command::new("explorer.exe").arg(&target).spawn()?;

        #[cfg(target_os = "macos")]
        Command::new("open").arg(&target).spawn()?;

        #[cfg(target_os = "linux")]
        Command::new("xdg-open").arg(&target).spawn()?;

        let display = format!(
            "{} {} {}",
            "✔ Opened directory".bright_green(),
            plain_path.cyan(),
            "in file explorer".bright_green()
        );
        return Ok((display, plain_path));
    }

    #[cfg(target_os = "windows")]
    {
        // 安全地打开文件：使用 Start-Process 的 -FilePath 指定路径，
        // 路径用英文双引号包裹，防止 PowerShell 解析空格、括号等特殊字符
        let path_str = target.to_string_lossy().to_string();
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Start-Process -FilePath \"{}\"",
                    path_str.replace('"', "`\""),
                ),
            ])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    Command::new("open").arg(&target).spawn()?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(&target).spawn()?;

    let display = format!(
        "{} {} {}",
        "✔ Opened file".bright_green(),
        plain_path.cyan(),
        "with default application".bright_green()
    );
    Ok((display, plain_path))
}
