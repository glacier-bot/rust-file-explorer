use colored::Color;
use std::path::Path;

/// 文件扩展名到图标和颜色的映射
const FILE_ICONS: &[(&[&str], &str, Color)] = &[
    (&["rs"], "🦀", Color::BrightRed),
    (&["toml", "json", "yaml", "yml"], "📋", Color::BrightYellow),
    (&["md", "txt"], "📝", Color::White),
    (&["gitignore", "git"], "🔀", Color::BrightMagenta),
    (&["exe", "bin"], "⚡", Color::BrightGreen),
    (&["jpg", "jpeg", "png", "gif", "svg"], "📷", Color::Magenta),
    (&["mp3", "wav", "flac"], "🎵", Color::BrightMagenta),
    (&["mp4", "avi", "mkv"], "🎬", Color::Red),
    (&["zip", "tar", "gz", "7z", "rar"], "📦", Color::BrightRed),
    (&["pdf"], "📕", Color::Red),
    (&["doc", "docx"], "📘", Color::BrightBlue),
    (&["xls", "xlsx"], "📗", Color::BrightGreen),
    (&["ppt", "pptx"], "📙", Color::BrightYellow),
    (&["html", "css", "js", "ts"], "🌐", Color::BrightCyan),
    (&["py"], "🐍", Color::BrightYellow),
    (&["go"], "🐹", Color::BrightCyan),
    (&["java"], "☕", Color::BrightRed),
    (&["c", "cpp", "h", "hpp"], "🔧", Color::BrightBlue),
    (&["sh", "bat", "ps1"], "💻", Color::BrightGreen),
    (&["lock"], "🔒", Color::BrightYellow),
    (&["log"], "📜", Color::BrightBlack),
];

pub fn get_file_icon_and_color(path: &Path, metadata: &std::fs::Metadata) -> (&'static str, Color) {
    if metadata.is_dir() {
        return ("📁", Color::BrightBlue);
    }
    if metadata.is_symlink() {
        return ("🔗", Color::Cyan);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    for &(exts, icon, color) in FILE_ICONS {
        if exts.contains(&ext.as_str()) {
            return (icon, color);
        }
    }

    ("📄", Color::White)
}
