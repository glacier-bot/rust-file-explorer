use arboard::Clipboard;
use colored::*;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Cmd, KeyCode, KeyEvent, Movement};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File, Metadata};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Instant, SystemTime};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// 目录内容缓存，用于加速补全
struct DirCache {
    entries: Vec<(String, bool)>, // (文件名, 是否目录)
    timestamp: Instant,
}

static DIR_CACHE: Mutex<Option<HashMap<String, DirCache>>> = Mutex::new(None);
const CACHE_TTL_MS: u64 = 5000; // 缓存有效期5秒

fn get_cached_dir_entries(path: &Path) -> Option<Vec<(String, bool)>> {
    let path_str = path.to_string_lossy().to_string();
    
    let mut cache_guard = DIR_CACHE.lock().ok()?;
    
    if cache_guard.is_none() {
        *cache_guard = Some(HashMap::new());
    }
    
    let cache = cache_guard.as_ref().unwrap();
    
    // 检查缓存是否有效
    if let Some(cached) = cache.get(&path_str) {
        if cached.timestamp.elapsed().as_millis() < CACHE_TTL_MS as u128 {
            return Some(cached.entries.clone());
        }
    }
    
    None
}

fn cache_dir_entries(path: &Path, entries: Vec<(String, bool)>) {
    let path_str = path.to_string_lossy().to_string();
    
    if let Ok(mut cache_guard) = DIR_CACHE.lock() {
        if let Some(ref mut cache) = cache_guard.as_mut() {
            cache.insert(path_str, DirCache {
                entries,
                timestamp: Instant::now(),
            });
        }
    }
}

struct FileInfo {
    name: String,
    icon: &'static str,
    color: Color,
    size: u64,
    created: Option<SystemTime>,
    modified: SystemTime,
    is_dir: bool,
    tags: Vec<String>,
}

struct AliasManager {
    aliases: HashMap<String, String>,
    config_path: PathBuf,
}

impl AliasManager {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("rfe");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("aliases.json");
        
        let mut aliases = HashMap::new();
        
        if config_path.exists() {
            let mut file = File::open(&config_path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            aliases = serde_json::from_str(&content)?;
        }
        
        Ok(Self { aliases, config_path })
    }
    
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.aliases)?;
        let mut file = File::create(&self.config_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
    
    fn add(&mut self, alias: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if alias.is_empty() {
            return Err("Alias cannot be empty".into());
        }
        if path.is_empty() {
            return Err("Path cannot be empty".into());
        }
        self.aliases.insert(alias.to_string(), path.to_string());
        self.save()?;
        Ok(())
    }
    
    fn remove(&mut self, alias: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.aliases.remove(alias).is_none() {
            return Err(format!("Alias '{}' does not exist", alias).into());
        }
        self.save()?;
        Ok(())
    }
    
    fn get(&self, alias: &str) -> Option<&String> {
        self.aliases.get(alias)
    }
    
    fn list(&self) -> &HashMap<String, String> {
        &self.aliases
    }
    
    fn resolve_path(&self, path: &str) -> String {
        if let Some(alias_part) = path.strip_prefix('@') {
            if let Some((alias_name, rest)) = alias_part.split_once('/') {
                if let Some(alias_path) = self.get(alias_name) {
                    return format!("{}/{}", alias_path, rest);
                }
            } else {
                if let Some(alias_path) = self.get(alias_part) {
                    return alias_path.clone();
                }
            }
        }
        path.to_string()
    }
    
    #[allow(dead_code)]
    fn resolve_all_paths(&self, args: &[&str]) -> Vec<String> {
        args.iter().map(|&s| self.resolve_path(s)).collect()
    }
}

struct TagManager {
    tags: HashMap<String, Vec<String>>,
    config_path: PathBuf,
    backup_path: PathBuf,
}

impl TagManager {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("rfe");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("tags.json");
        let backup_path = config_dir.join("tags.json.bak");
        
        let mut tags = HashMap::new();
        
        if config_path.exists() {
            match Self::load_from_file(&config_path) {
                Ok(loaded_tags) => tags = loaded_tags,
                Err(e) => {
                    // 如果主文件损坏，尝试从备份恢复
                    if backup_path.exists() {
                        match Self::load_from_file(&backup_path) {
                            Ok(backup_tags) => {
                                tags = backup_tags;
                                // 恢复主文件
                                fs::copy(&backup_path, &config_path)?;
                                eprintln!("⚠️  标签主文件损坏，已从备份恢复");
                            }
                            Err(_) => return Err(format!("标签文件损坏且备份也无法读取: {}", e).into())
                        }
                    } else {
                        return Err(format!("标签文件损坏且无备份可用: {}", e).into())
                    }
                }
            }
        } else if backup_path.exists() {
            // 主文件不存在但有备份，恢复
            if let Ok(backup_tags) = Self::load_from_file(&backup_path) {
                tags = backup_tags;
                fs::copy(&backup_path, &config_path)?;
                eprintln!("⚠️  标签主文件丢失，已从备份恢复");
            }
        }
        
        Ok(Self { tags, config_path, backup_path })
    }
    
    fn load_from_file(path: &PathBuf) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(serde_json::from_str(&content)?)
    }
    
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 原子性写入：先写临时文件，再替换原文件，避免写入中断导致文件损坏
        let temp_path = self.config_path.with_extension("tmp");
        let content = serde_json::to_string_pretty(&self.tags)?;
        
        let mut file = File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        
        // 备份当前文件
        if self.config_path.exists() {
            fs::copy(&self.config_path, &self.backup_path)?;
        }
        
        // 替换原文件
        fs::rename(&temp_path, &self.config_path)?;
        
        Ok(())
    }
    
    // 手动备份
    fn backup(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config_path.exists() {
            fs::copy(&self.config_path, &self.backup_path)?;
            Ok(())
        } else {
            Err("标签文件不存在，无法备份".into())
        }
    }
    
    // 从备份恢复
    fn restore(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.backup_path.exists() {
            self.tags = Self::load_from_file(&self.backup_path)?;
            self.save()?;
            Ok(())
        } else {
            Err("备份文件不存在，无法恢复".into())
        }
    }
    
    // 为文件添加标签，支持多个标签，自动去重
    fn add_tags(&mut self, file_path: &str, tags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err(format!("文件或文件夹不存在: {}", file_path).into());
        }
        
        // 转换为绝对路径
        let abs_path = fs::canonicalize(&path)?
            .to_string_lossy()
            .to_string();
        
        // 验证标签有效性，不能包含空格、特殊字符
        for tag in tags {
            if tag.is_empty() {
                return Err("标签不能为空".into());
            }
            if tag.contains(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '/' || c == '\\') {
                return Err(format!("标签包含无效字符: {}", tag).into());
            }
        }
        
        let existing_tags = self.tags.entry(abs_path).or_default();
        for tag in tags {
            let tag_str = tag.to_string();
            if !existing_tags.contains(&tag_str) {
                existing_tags.push(tag_str);
            }
        }
        
        self.save()?;
        Ok(())
    }
    
    // 移除文件的指定标签，如果标签为空则删除该文件的标签记录
    fn remove_tags(&mut self, file_path: &str, tags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err(format!("文件或文件夹不存在: {}", file_path).into());
        }
        
        let abs_path = fs::canonicalize(&path)?
            .to_string_lossy()
            .to_string();
        
        if let Some(existing_tags) = self.tags.get_mut(&abs_path) {
            for tag in tags {
                existing_tags.retain(|t| t != tag);
            }
            
            if existing_tags.is_empty() {
                self.tags.remove(&abs_path);
            }
            
            self.save()?;
            Ok(())
        } else {
            Err(format!("该文件没有任何标签: {}", file_path).into())
        }
    }
    
    // 移除文件的所有标签
    fn remove_all_tags(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err(format!("文件或文件夹不存在: {}", file_path).into());
        }
        
        let abs_path = fs::canonicalize(&path)?
            .to_string_lossy()
            .to_string();
        
        if self.tags.remove(&abs_path).is_some() {
            self.save()?;
            Ok(())
        } else {
            Err(format!("该文件没有任何标签: {}", file_path).into())
        }
    }
    
    // 获取文件的所有标签
    fn get_tags(&self, file_path: &str) -> Vec<String> {
        match fs::canonicalize(file_path) {
            Ok(abs_path) => {
                let path_str = abs_path.to_string_lossy().to_string();
                self.tags.get(&path_str).cloned().unwrap_or_default()
            }
            Err(_) => Vec::new()
        }
    }
    
    // 获取所有标签列表，用于自动补全
    fn get_all_tags(&self) -> Vec<String> {
        let mut all_tags = std::collections::HashSet::new();
        for tags in self.tags.values() {
            for tag in tags {
                all_tags.insert(tag.clone());
            }
        }
        all_tags.into_iter().collect()
    }
    
    // 列出所有带标签的文件及其标签
    fn list_all(&self) -> &HashMap<String, Vec<String>> {
        &self.tags
    }
    
    // 检查文件是否匹配所有标签正则条件
    fn file_matches_tags(&self, file_path: &str, tag_patterns: &[Regex]) -> bool {
        match fs::canonicalize(file_path) {
            Ok(abs_path) => {
                let path_str = abs_path.to_string_lossy().to_string();
                match self.tags.get(&path_str) {
                    Some(tags) => {
                        // 所有标签正则都必须匹配至少一个文件标签
                        tag_patterns.iter().all(|pattern| {
                            tags.iter().any(|tag| pattern.is_match(tag))
                        })
                    }
                    None => false
                }
            }
            Err(_) => false
        }
    }
    
    // 全局搜索所有匹配标签正则的文件
    fn find_files_by_tags(&self, tag_patterns: &[Regex]) -> Vec<(String, Vec<String>)> {
        let mut result = Vec::new();
        for (path, tags) in &self.tags {
            // 检查是否匹配所有正则
            if tag_patterns.iter().all(|pattern| {
                tags.iter().any(|tag| pattern.is_match(tag))
            }) {
                // 处理路径格式
                let mut display_path = path.clone();
                if cfg!(windows) && display_path.starts_with("\\\\?\\") {
                    display_path = display_path[4..].to_string();
                }
                result.push((display_path, tags.clone()));
            }
        }
        result
    }
}

struct RfeHelper {
    completer: FilenameCompleter,
    alias_manager: AliasManager,
    tag_manager: TagManager,
}

impl Completer for RfeHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let current_word = &line[..pos];
        
        // 路径别名补全 - 支持 @alias/path 的层级补全
        if let Some(at_pos) = current_word.rfind('@') {
            let after_at = &current_word[at_pos + 1..];
            
            // 检查是否包含路径分隔符（需要子路径补全）
            if let Some(sep_pos) = after_at.find('/') {
                let alias_name = &after_at[..sep_pos];
                let sub_path = &after_at[sep_pos + 1..];
                
                // 获取别名对应的真实路径
                if let Some(alias_path) = self.alias_manager.get(alias_name) {
                    let base_path = PathBuf::from(alias_path);
                    
                    // 解析子路径，确定要浏览的目录
                    let (dir_to_list, file_prefix) = if sub_path.ends_with('/') {
                        (base_path.join(sub_path), "")
                    } else if let Some(last_sep) = sub_path.rfind('/') {
                        let dir_part = &sub_path[..last_sep];
                        let file_part = &sub_path[last_sep + 1..];
                        (base_path.join(dir_part), file_part)
                    } else {
                        (base_path.clone(), sub_path)
                    };
                    
                    // 读取目录内容并提供补全（带缓存和性能限制）
                    if dir_to_list.is_dir() {
                        let start_time = Instant::now();
                        let mut candidates = Vec::new();
                        const MAX_COMPLETION_TIME_MS: u128 = 100; // 单次补全最大耗时100ms
                        const MAX_ENTRIES: usize = 100; // 最大补全条目数
                        
                        // 尝试从缓存获取
                        let entries: Vec<(String, bool)> = if let Some(cached) = get_cached_dir_entries(&dir_to_list) {
                            cached
                        } else {
                            // 读取目录并缓存
                            let mut new_entries = Vec::new();
                            if let Ok(dir_entries) = fs::read_dir(&dir_to_list) {
                                for entry in dir_entries.filter_map(|e| e.ok()) {
                                    if let Some(name) = entry.file_name().to_str() {
                                        let is_dir = entry.metadata().ok()
                                            .map(|m| m.is_dir()).unwrap_or(false);
                                        new_entries.push((name.to_string(), is_dir));
                                    }
                                }
                            }
                            cache_dir_entries(&dir_to_list, new_entries.clone());
                            new_entries
                        };
                        
                        // 生成补全候选
                        for (name, is_dir) in entries {
                            // 性能检查：超时则返回已有结果
                            if start_time.elapsed().as_millis() > MAX_COMPLETION_TIME_MS {
                                break;
                            }
                            
                            // 过滤匹配前缀的条目
                            if !file_prefix.is_empty() && !name.starts_with(file_prefix) {
                                continue;
                            }
                            
                            // 限制最大条目数
                            if candidates.len() >= MAX_ENTRIES {
                                break;
                            }
                            
                            // 构建补全路径
                            let replacement = if let Some(last_sep) = sub_path.rfind('/') {
                                format!("@{}/{}/{}", alias_name, &sub_path[..last_sep], name)
                            } else {
                                format!("@{}/{}", alias_name, name)
                            };
                            
                            // 如果是目录，添加尾部斜杠
                            let replacement_with_sep = if is_dir {
                                format!("{}/", replacement)
                            } else {
                                replacement.clone()
                            };
                            
                            candidates.push(Pair {
                                display: name.clone(),
                                replacement: replacement_with_sep,
                            });
                        }
                        
                        // 按目录在前、文件在后排序
                        candidates.sort_by(|a, b| {
                            let a_is_dir = a.replacement.ends_with('/');
                            let b_is_dir = b.replacement.ends_with('/');
                            match (a_is_dir, b_is_dir) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.display.cmp(&b.display),
                            }
                        });
                        
                        if !candidates.is_empty() {
                            return Ok((at_pos, candidates));
                        }
                    }
                }
            } else {
                // 纯别名补全（无子路径）
                let alias_prefix = after_at;
                let mut candidates = Vec::new();
                
                for (alias, path) in self.alias_manager.list() {
                    if alias.starts_with(alias_prefix) {
                        candidates.push(Pair {
                            display: format!("📍 @{} -> {}", alias, path),
                            replacement: format!("@{}", alias),
                        });
                    }
                }
                
                // 如果有匹配的别名，同时提供别名的子路径补全
                if candidates.len() == 1 || alias_prefix.is_empty() {
                    // 获取第一个匹配别名的目录内容作为额外补全
                    for (alias, path) in self.alias_manager.list() {
                        if alias.starts_with(alias_prefix) {
                            let alias_path = PathBuf::from(path);
                            if alias_path.is_dir() {
                                // 使用缓存获取目录内容
                                let entries = if let Some(cached) = get_cached_dir_entries(&alias_path) {
                                    cached
                                } else {
                                    let mut new_entries = Vec::new();
                                    if let Ok(dir_entries) = fs::read_dir(&alias_path) {
                                        for entry in dir_entries.filter_map(|e| e.ok()) {
                                            if let Some(name) = entry.file_name().to_str() {
                                                let is_dir = entry.metadata().ok()
                                                    .map(|m| m.is_dir()).unwrap_or(false);
                                                new_entries.push((name.to_string(), is_dir));
                                            }
                                        }
                                    }
                                    cache_dir_entries(&alias_path, new_entries.clone());
                                    new_entries
                                };
                                
                                let mut sub_candidates = Vec::new();
                                for (name, is_dir) in entries.into_iter().take(20) {
                                    let replacement = if is_dir {
                                        format!("@{}/{}/", alias, name)
                                    } else {
                                        format!("@{}/{}", alias, name)
                                    };
                                    
                                    sub_candidates.push(Pair {
                                        display: name,
                                        replacement,
                                    });
                                }
                                candidates.extend(sub_candidates);
                            }
                            break; // 只处理第一个匹配的别名
                        }
                    }
                }
                
                if !candidates.is_empty() {
                    return Ok((at_pos, candidates));
                }
            }
        }
        
        // 标签补全：当命令是tag add/tag remove时补全标签名
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && (parts[0] == "tag" || parts[0] == "t") {
            match parts[1] {
                "add" | "remove" | "rm" if parts.len() >= 3 => {
                    // 当前正在输入标签
                    let tag_prefix = current_word.split_whitespace().last().unwrap_or("");
                    let mut candidates = Vec::new();
                    
                    for tag in self.tag_manager.get_all_tags() {
                        if tag.starts_with(tag_prefix) {
                            candidates.push(Pair {
                                display: tag.clone(),
                                replacement: tag,
                            });
                        }
                    }
                    
                    if !candidates.is_empty() {
                        let start_pos = pos - tag_prefix.len();
                        return Ok((start_pos, candidates));
                    }
                }
                _ => {}
            }
        }
        
        self.completer.complete(line, pos, ctx)
    }
}

impl Helper for RfeHelper {}
impl Highlighter for RfeHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> std::borrow::Cow<'b, str> {
        if prompt.starts_with("rfe ") && prompt.ends_with(" >") {
            let dir = &prompt[4..prompt.len() - 2];
            let colored = format!(
                "{} {} {}",
                "rfe".bright_green().bold(),
                dir.bright_blue().bold(),
                ">".bright_blue().bold()
            );
            std::borrow::Cow::Owned(colored)
        } else {
            std::borrow::Cow::Borrowed(prompt)
        }
    }
}
impl Hinter for RfeHelper {
    type Hint = String;
}
impl Validator for RfeHelper {}

fn get_file_icon_and_color(path: &Path, metadata: &Metadata) -> (&'static str, Color) {
    if metadata.is_dir() {
        ("📁", Color::BrightBlue)
    } else if metadata.is_symlink() {
        ("🔗", Color::Cyan)
    } else {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => ("🦀", Color::BrightRed),
            "toml" | "json" | "yaml" | "yml" => ("📋", Color::BrightYellow),
            "md" | "txt" => ("📝", Color::White),
            "gitignore" | "git" => ("🔀", Color::BrightMagenta),
            "exe" | "bin" => ("⚡", Color::BrightGreen),
            "jpg" | "jpeg" | "png" | "gif" | "svg" => ("📷", Color::Magenta),
            "mp3" | "wav" | "flac" => ("🎵", Color::BrightMagenta),
            "mp4" | "avi" | "mkv" => ("🎬", Color::Red),
            "zip" | "tar" | "gz" | "7z" | "rar" => ("📦", Color::BrightRed),
            "pdf" => ("📕", Color::Red),
            "doc" | "docx" => ("📘", Color::BrightBlue),
            "xls" | "xlsx" => ("📗", Color::BrightGreen),
            "ppt" | "pptx" => ("📙", Color::BrightYellow),
            "html" | "css" | "js" | "ts" => ("🌐", Color::BrightCyan),
            "py" => ("🐍", Color::BrightYellow),
            "go" => ("🐹", Color::BrightCyan),
            "java" => ("☕", Color::BrightRed),
            "c" | "cpp" | "h" | "hpp" => ("🔧", Color::BrightBlue),
            "sh" | "bat" | "ps1" => ("💻", Color::BrightGreen),
            "lock" => ("🔒", Color::BrightYellow),
            "log" => ("📜", Color::BrightBlack),
            _ => ("📄", Color::White),
        }
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:>5.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:>5.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:>5.1} KB", size as f64 / KB as f64)
    } else {
        format!("{:>6} B", size)
    }
}

fn format_time_absolute(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let total_secs = duration.as_secs();
            let mut days = total_secs / 86400;
            let secs_in_day = total_secs % 86400;
            let hours = secs_in_day / 3600;
            let mins = (secs_in_day % 3600) / 60;
            let secs = secs_in_day % 60;

            let mut year = 1970;
            while days >= if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                366
            } else {
                365
            } {
                days -= if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    366
                } else {
                    365
                };
                year += 1;
            }

            let mut month = 1;
            let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            for &md in month_days.iter() {
                let adjust = if month == 2 && (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) {
                    1
                } else {
                    0
                };
                if days < md + adjust {
                    break;
                }
                days -= md + adjust;
                month += 1;
            }

            let day = days + 1;

            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                year, month, day, hours, mins, secs
            )
        }
        Err(_) => "                   N/A".to_string(),
    }
}

fn truncate_string(s: &str, max_width: usize) -> String {
    let width = s.width();
    if width <= max_width {
        return s.to_string();
    }

    let available_width = max_width.saturating_sub(3);
    if available_width == 0 {
        return "...".to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let c_width = c.width().unwrap_or(1);
        if current_width + c_width > available_width {
            break;
        }
        result.push(c);
        current_width += c_width;
    }

    result + "..."
}

fn get_terminal_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(120)
}

fn make_separator(widths: &[usize]) -> String {
    let mut result = String::new();
    for &w in widths {
        result.push('+');
        result.push_str(&"-".repeat(w + 2));
    }
    result.push('+');
    result
}

fn center_text(text: &str, width: usize) -> String {
    let text_width = text.width();
    if text_width >= width {
        return truncate_string(text, width);
    }
    let left = (width - text_width) / 2;
    let right = width - text_width - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}

fn pad_to_width(s: &str, width: usize) -> String {
    let s_width = s.width();
    if s_width >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s_width))
    }
}

fn calculate_column_widths(term_width: usize, show_tags: bool) -> (usize, usize, usize, usize, usize) {
    let min_name = 10;
    let min_date = 10;
    let min_size = 6;
    let min_tags = 5;

    let border_chars = if show_tags { 16 } else { 13 };
    let content_total = term_width.saturating_sub(border_chars);
    let min_total = min_name + min_date * 2 + min_size + if show_tags { min_tags } else { 0 };

    if content_total < min_total {
        let tags_w = if show_tags { min_tags } else { 0 };
        return (min_name, min_date, min_date, min_size, tags_w);
    }

    if show_tags {
        let total_parts = 118;
        let name = (content_total * 41 / total_parts).max(min_name);
        let created = (content_total * 21 / total_parts).max(min_date);
        let modified = (content_total * 21 / total_parts).max(min_date);
        let size = (content_total * 12 / total_parts).max(min_size);
        let tags = (content_total * 23 / total_parts).max(min_tags);

        let total = name + created + modified + size + tags;
        if total > content_total {
            let excess = total - content_total;
            let name = name.saturating_sub(excess).max(min_name);
            (name, created, modified, size, tags)
        } else {
            (name, created, modified, size, tags)
        }
    } else {
        let total_parts = 95;
        let name = (content_total * 41 / total_parts).max(min_name);
        let created = (content_total * 21 / total_parts).max(min_date);
        let modified = (content_total * 21 / total_parts).max(min_date);
        let size = (content_total * 12 / total_parts).max(min_size);

        let total = name + created + modified + size;
        if total > content_total {
            let excess = total - content_total;
            let name = name.saturating_sub(excess).max(min_name);
            (name, created, modified, size, 0)
        } else {
            (name, created, modified, size, 0)
        }
    }
}

fn is_hidden(path: &PathBuf) -> bool {
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
        match fs::metadata(path) {
            Ok(meta) => (meta.file_attributes() & 2) != 0,
            Err(_) => false,
        }
    }
}

fn print_welcome() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════╗".bright_green()
    );
    println!(
        "{}",
        "║           Rust File Explorer v0.3.0                          ║".bright_green()
    );
    println!(
        "{}",
        "║           A cross-platform CLI file browser                  ║".bright_green()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════╝".bright_green()
    );
    println!();
    println!("{}", "Commands:".bright_yellow());
    println!("  {}  - List directory contents (supports --re regex search, -t tag filter)", "ls".cyan());
    println!("  {}  - Print working directory", "pwd".cyan());
    println!("  {}  - Copy current directory path to clipboard", "cppwd".cyan());
    println!("  {}  - Copy file absolute path to clipboard", "cpf <file>".cyan());
    println!("  {}  - Change directory", "cd <path>".cyan());
    println!("  {}  - Open file with default application / Open directory in file explorer", "open <path>".cyan());
    println!("  {}  - Move/copy file or folder (use --cp to copy)", "mv <source> <dest> [--cp]".cyan());
    println!("  {}  - Create file or directory (-f for file, -d for directory)", "mkdf".cyan());
    println!("  {}  - Manage path aliases (add/remove/list)", "alias".cyan());
    println!("  {}  - Manage file tags (add/remove/find/list/backup)", "tag".cyan());
    println!("  {}  - Exit the program", "exit".cyan());
    println!("  {} - Clear the screen", "clear".cyan());
    println!("  {}  - Show this help", "help".cyan());
    println!();
    println!("{}", "Keyboard shortcuts:".bright_yellow());
    println!("  {} - Clear current input line in REPL mode", "ESC".cyan());
    println!();
}

fn get_prompt_string() -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let dir_str = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");

    format!("rfe {} >", dir_str)
}

fn cmd_pwd() -> Result<(String, String), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    let plain_path = current_dir.display().to_string();
    let display_output = plain_path.bright_cyan().to_string();
    Ok((display_output, plain_path))
}

fn cmd_cppwd() -> Result<(String, String), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let path_str = cwd.to_string_lossy().to_string();

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("failed to access system clipboard: {}", e))?;
    clipboard
        .set_text(&path_str)
        .map_err(|e| format!("failed to copy to clipboard: {}", e))?;

    let display = format!("{} {}", "✔ Copied to clipboard:".bright_green(), path_str.cyan());
    Ok((display, path_str))
}

fn cmd_cpf(file_path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let path = PathBuf::from(file_path);

    if !path.exists() {
        return Err(format!("file does not exist: {}", file_path).into());
    }

    if !path.is_file() {
        return Err(format!("not a file: {}", file_path).into());
    }

    let abs_path = fs::canonicalize(&path)
        .map_err(|e| format!("failed to resolve absolute path: {}", e))?;

    let path_str = if cfg!(windows) {
        let s = abs_path.to_string_lossy().to_string();
        s.strip_prefix(r"\\?\").map(|stripped| stripped.to_string()).unwrap_or(s)
    } else {
        abs_path.to_string_lossy().to_string()
    };

    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("failed to access system clipboard: {}", e))?;
    clipboard
        .set_text(&path_str)
        .map_err(|e| format!("failed to copy to clipboard: {}", e))?;

    let display = format!("{} {}", "✔ Copied to clipboard:".bright_green(), path_str.cyan());
    Ok((display, path_str))
}

fn cmd_cd(path: Option<&str>) -> Result<(String, String), Box<dyn std::error::Error>> {
    let target = match path {
        Some("..") => {
            let mut current = env::current_dir()?;
            current.pop();
            current
        }
        Some("~") | None => dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        Some(p) => PathBuf::from(p),
    };

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    if !target.is_dir() {
        return Err(format!("Not a directory: {}", target.display()).into());
    }

    env::set_current_dir(&target)?;
    let plain_path = target.display().to_string();
    let display = format!("{} {}", "Changed to:".green(), plain_path.cyan());
    Ok((display, plain_path))
}

use regex::Regex;

#[allow(clippy::too_many_arguments)]
fn cmd_ls(all: bool, long: bool, re: bool, re_insensitive: bool, show_tags: bool, recursive: bool, path: Option<&str>, tag_manager: &TagManager, tag_patterns: &[Regex]) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut output = String::new();
    let mut files: Vec<FileInfo> = Vec::new();
    let mut dirs: Vec<FileInfo> = Vec::new();

    if re {
        let pattern = path.ok_or("Regex pattern required when using --re flag")?;
        
        let re_pattern = if re_insensitive {
            Regex::new(&format!("(?i){}", pattern))
        } else {
            Regex::new(pattern)
        }.map_err(|e| format!("Invalid regular expression: {}", e))?;

        let search_dir = if pattern.starts_with('/') || (cfg!(windows) && pattern.contains(':')) {
            PathBuf::from("/")
        } else {
            env::current_dir()?
        };

        output.push_str(&format!(
            "{} {}\n\n",
            "🔍 Regex Search:".bright_yellow().bold(),
            pattern.bright_cyan()
        ));

        fn walk_dir(dir: &Path, pattern: &Regex, all: bool, show_tags: bool, recursive: bool, tag_manager: &TagManager, files: &mut Vec<FileInfo>, dirs: &mut Vec<FileInfo>) -> Result<(), Box<dyn std::error::Error>> {
            for entry in fs::read_dir(dir)?.filter_map(|e| e.ok()) {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                if !all && is_hidden(&path) {
                    continue;
                }

                if pattern.is_match(&path_str) {
                    match entry.metadata() {
                        Ok(meta) => {
                            let (icon, color) = get_file_icon_and_color(&path, &meta);
                            let created = meta.created().ok();
                            let modified = meta.modified().unwrap_or_else(|_| SystemTime::now());
                            let name = path.strip_prefix(env::current_dir()?)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .to_string();

                            let tags = if show_tags {
                                tag_manager.get_tags(path.to_str().unwrap_or(""))
                            } else {
                                Vec::new()
                            };
                            
                            let file_info = FileInfo {
                                name,
                                icon,
                                color,
                                size: meta.len(),
                                created,
                                modified,
                                is_dir: meta.is_dir(),
                                tags,
                            };

                            if meta.is_dir() {
                                dirs.push(file_info);
                            } else {
                                files.push(file_info);
                            }
                        }
                        Err(_) => continue,
                    }
                }

                if path.is_dir() && recursive && walk_dir(&path, pattern, all, show_tags, recursive, tag_manager, files, dirs).is_err() {
                    continue;
                }
            }
            Ok(())
        }

        walk_dir(&search_dir, &re_pattern, all, show_tags, recursive, tag_manager, &mut files, &mut dirs)?
    } else {
        let target = match path {
            Some(p) => PathBuf::from(p),
            None => env::current_dir()?,
        };

        if !target.exists() {
            return Err(format!("Path does not exist: {}", target.display()).into());
        }

        output.push_str(&format!(
            "{} {}\n\n",
            "📂 Directory:".bright_yellow().bold(),
            target.display().to_string().bright_cyan()
        ));

        let entries = fs::read_dir(target)?;

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if !all && is_hidden(&path) {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();

            match entry.metadata() {
                Ok(meta) => {
                    let (icon, color) = get_file_icon_and_color(&path, &meta);
                    let created = meta.created().ok();
                    let modified = meta.modified().unwrap_or_else(|_| SystemTime::now());

                    let tags = if show_tags {
                        tag_manager.get_tags(path.to_str().unwrap_or(""))
                    } else {
                        Vec::new()
                    };
                    
                    let file_info = FileInfo {
                        name,
                        icon,
                        color,
                        size: meta.len(),
                        created,
                        modified,
                        is_dir: meta.is_dir(),
                        tags,
                    };

                    if meta.is_dir() {
                        dirs.push(file_info);
                    } else {
                        files.push(file_info);
                    }
                }
                Err(_) => {
                    let file_info = FileInfo {
                        name,
                        icon: "❓",
                        color: Color::Red,
                        size: 0,
                        created: None,
                        modified: SystemTime::now(),
                        is_dir: false,
                        tags: Vec::new(),
                    };
                    files.push(file_info);
                }
            }
        }
    }

    dirs.sort_by_key(|a| a.name.to_lowercase());
    files.sort_by_key(|a| a.name.to_lowercase());

    let mut all_items = Vec::new();
    all_items.extend(dirs);
    all_items.extend(files);
    
    // 按标签过滤
    if !tag_patterns.is_empty() {
        all_items.retain(|item| {
            // 构造文件完整路径
            let full_path = match &path {
                Some(p) => Path::new(p).join(&item.name),
                None => env::current_dir().unwrap_or_default().join(&item.name),
            };
            tag_manager.file_matches_tags(full_path.to_str().unwrap_or_default(), tag_patterns)
        });
    }

    if long {
        let term_width = get_terminal_width();
        let (name_width, created_width, modified_width, size_width, tags_width) =
            calculate_column_widths(term_width, show_tags);
        let truncate_name_width = name_width.saturating_sub(4);

        if show_tags {
            let widths = [name_width, created_width, modified_width, size_width, tags_width];
            let separator = make_separator(&widths).bright_black();

            output.push_str(&format!("{}\n", separator));
            output.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                center_text("Name", name_width).bright_white().bold(),
                center_text("Created Date", created_width).bright_white().bold(),
                center_text("Modified Date", modified_width).bright_white().bold(),
                center_text("Size", size_width).bright_white().bold(),
                center_text("Tags", tags_width).bright_white().bold(),
            ));
            output.push_str(&format!("{}\n", separator));

            for item in &all_items {
                let created_str = item
                    .created
                    .map_or("N/A".to_string(), format_time_absolute);
                let modified_str = format_time_absolute(item.modified);
                let tags_str = if item.tags.is_empty() {
                    "".to_string()
                } else {
                    item.tags.join(", ")
                };

                let display_name = truncate_string(&item.name, truncate_name_width);
                let display_text = format!("{}  {}", item.icon, display_name);
                let display_text_width = display_text.width();
                let padding = if display_text_width < name_width {
                    " ".repeat(name_width - display_text_width)
                } else {
                    String::new()
                };
                let padded_name = format!("{}{}", display_text, padding);
                let padded_tags = pad_to_width(&truncate_string(&tags_str, tags_width), tags_width);

                output.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    padded_name.color(item.color).bold(),
                    pad_to_width(&truncate_string(&created_str, created_width), created_width).bright_cyan(),
                    pad_to_width(&truncate_string(&modified_str, modified_width), modified_width).bright_magenta(),
                    pad_to_width(&format_size(item.size), size_width).bright_yellow().bold(),
                    padded_tags.bright_yellow()
                ));
            }

            output.push_str(&format!("{}\n", separator));
        } else {
            let widths = [name_width, created_width, modified_width, size_width];
            let separator = make_separator(&widths).bright_black();

            output.push_str(&format!("{}\n", separator));
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                center_text("Name", name_width).bright_white().bold(),
                center_text("Created Date", created_width).bright_white().bold(),
                center_text("Modified Date", modified_width).bright_white().bold(),
                center_text("Size", size_width).bright_white().bold(),
            ));
            output.push_str(&format!("{}\n", separator));

            for item in &all_items {
                let created_str = item
                    .created
                    .map_or("N/A".to_string(), format_time_absolute);
                let modified_str = format_time_absolute(item.modified);

                let display_name = truncate_string(&item.name, truncate_name_width);
                let display_text = format!("{}  {}", item.icon, display_name);
                let display_text_width = display_text.width();
                let padding = if display_text_width < name_width {
                    " ".repeat(name_width - display_text_width)
                } else {
                    String::new()
                };
                let padded_name = format!("{}{}", display_text, padding);

                output.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    padded_name.color(item.color).bold(),
                    pad_to_width(&truncate_string(&created_str, created_width), created_width).bright_cyan(),
                    pad_to_width(&truncate_string(&modified_str, modified_width), modified_width).bright_magenta(),
                    pad_to_width(&format_size(item.size), size_width).bright_yellow().bold()
                ));
            }

            output.push_str(&format!("{}\n", separator));
        }
    } else {
        for item in &all_items {
            let display_name = truncate_string(&item.name, 50);
            if show_tags && !item.tags.is_empty() {
                let tags_str = format!(" [{}]", item.tags.join(", "));
                output.push_str(&format!("  {} {}{}\n", item.icon, display_name.color(item.color).bold(), tags_str.bright_yellow()));
            } else {
                output.push_str(&format!("  {} {}\n", item.icon, display_name.color(item.color).bold()));
            }
        }
    }

    output.push('\n');
    let total = all_items.len();
    let dir_count = all_items.iter().filter(|f| f.is_dir).count();
    let file_count = total - dir_count;

    output.push_str(&format!(
        "{} {} directories, {} files\n\n",
        "📊".bright_green(),
        dir_count.to_string().bright_blue(),
        file_count.to_string().bright_cyan()
    ));
    
    // Get raw path for piping
    let raw_path = if re {
        let search_dir = match path {
            Some(p) if p.starts_with('/') || (cfg!(windows) && p.contains(':')) => PathBuf::from("/"),
            _ => env::current_dir()?
        };
        search_dir.display().to_string()
    } else {
        match path {
            Some(p) => PathBuf::from(p).display().to_string(),
            None => env::current_dir()?.display().to_string()
        }
    };

    Ok((output, raw_path))
}

fn cmd_open(path: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let target = PathBuf::from(path);

    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    let plain_path = target.display().to_string();
    
    if target.is_dir() {
        #[cfg(target_os = "windows")]
        Command::new("explorer.exe")
            .arg(&target)
            .spawn()?;
        
        #[cfg(target_os = "macos")]
        Command::new("open")
            .arg(&target)
            .spawn()?;
        
        #[cfg(target_os = "linux")]
        Command::new("xdg-open")
            .arg(&target)
            .spawn()?;
        
        let display = format!(
            "{} {} {}",
            "✔ Opened directory".bright_green(),
            plain_path.cyan(),
            "in file explorer".bright_green()
        );
        return Ok((display, plain_path));
    }

    open::that(&target)?;
    let display = format!(
        "{} {} {}",
        "✔ Opened file".bright_green(),
        plain_path.cyan(),
        "with default application".bright_green()
    );
    Ok((display, plain_path))
}

fn cmd_clear() -> Result<(String, String), Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush()?;
    Ok((String::new(), String::new()))
}

fn cmd_mkdf(args: &[&str]) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut create_file = false;
    let mut create_dir = false;
    let mut parents = false;
    let mut path: Option<String> = None;
    let mut show_help = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-f" | "--file" => {
                create_file = true;
                create_dir = false;
            }
            "-d" | "--directory" => {
                create_dir = true;
                create_file = false;
            }
            "-p" | "--parents" => {
                parents = true;
            }
            "-h" | "--help" => {
                show_help = true;
            }
            p => {
                if path.is_none() {
                    path = Some(p.to_string());
                } else {
                    return Err("Too many arguments. Only one path can be specified.".into());
                }
            }
        }
        i += 1;
    }
    
    if show_help {
        let mut output = format!("{}\n\n", "📁 mkdf Command Help:".bright_yellow().bold());
        output.push_str(&format!("  {} Create a file\n", "mkdf -f/--file <path>".cyan().bold()));
        output.push_str(&format!("  {} Create a directory\n", "mkdf -d/--directory <path>".cyan().bold()));
        output.push_str(&format!("  {} Create parent directories if they don't exist\n", "mkdf -p/--parents".cyan().bold()));
        output.push_str(&format!("  {} Show this help\n\n", "mkdf -h/--help".cyan().bold()));
        output.push_str(&format!("{}\n", "Examples:".bright_green().bold()));
        output.push_str(&format!("  {} Create a file named 'test.txt'\n", "mkdf -f test.txt".cyan()));
        output.push_str(&format!("  {} Create a directory named 'new_folder'\n", "mkdf -d new_folder".cyan()));
        output.push_str(&format!("  {} Create a file with parent directories\n", "mkdf -f -p path/to/file.txt".cyan()));
        output.push_str(&format!("  {} Create nested directories\n", "mkdf -d -p parent/child/grandchild".cyan()));
        return Ok((output, String::new()));
    }
    
    let path = path.ok_or("Usage: mkdf [-f|--file|-d|--directory] [-p|--parents] <path>")?;
    
    if !create_file && !create_dir {
        return Err("Please specify whether to create a file (-f/--file) or directory (-d/--directory)".into());
    }
    
    let target_path = PathBuf::from(&path);
    
    if target_path.exists() {
        return Err(format!("Path already exists: {}", target_path.display()).into());
    }
    
    if create_file {
        // 创建文件时，自动创建父目录
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        File::create(&target_path)?;
        Ok((format!("{} Created file: {}", "✔".bright_green(), target_path.display().to_string().cyan()), target_path.display().to_string()))
    } else {
        // 创建目录时，根据是否指定了-p参数决定是否创建父目录
        if parents {
            fs::create_dir_all(&target_path)?;
            Ok((format!("{} Created directory (with parents): {}", "✔".bright_green(), target_path.display().to_string().cyan()), target_path.display().to_string()))
        } else {
            // 不使用-p参数时，只创建最后一级目录，父目录必须存在
            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    return Err(format!("Parent directory does not exist. Use -p/--parents to create it: {}", parent.display()).into());
                }
            }
            
            fs::create_dir(&target_path)?;
            Ok((format!("{} Created directory: {}", "✔".bright_green(), target_path.display().to_string().cyan()), target_path.display().to_string()))
        }
    }
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let entry_path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if entry_type.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&entry_path)?;
                fs::set_permissions(&dest_path, metadata.permissions())?;
            }
            
            #[cfg(windows)]
            {

                let metadata = fs::metadata(&entry_path)?;
                let mut perm = fs::metadata(&dest_path)?.permissions();
                perm.set_readonly(metadata.permissions().readonly());
                fs::set_permissions(&dest_path, perm)?;
            }
        }
    }

    Ok(())
}

fn cmd_mv(source: &str, destination: &str, copy: bool) -> Result<(String, String), Box<dyn std::error::Error>> {
    let source_path = PathBuf::from(source);
    let dest_path = PathBuf::from(destination);

    if !source_path.exists() {
        return Err(format!("Source path does not exist: {}", source_path.display()).into());
    }

    let source_metadata = fs::metadata(&source_path)?;

    let final_dest = if dest_path.is_dir() {
        dest_path.join(source_path.file_name().ok_or("Invalid source path")?)
    } else {
        dest_path.clone()
    };

    if final_dest.exists() {
        return Err(format!("Destination path already exists: Please remove it first: {}", final_dest.display()).into());
    }

    let output = if copy {
        if source_metadata.is_dir() {
            copy_dir_recursive(&source_path, &final_dest)?;
            format!(
                "{} Copied directory {} to {}",
                "✔".bright_green(),
                source_path.display().to_string().cyan(),
                final_dest.display().to_string().cyan()
            )
        } else {
            fs::copy(&source_path, &final_dest)?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perm = fs::metadata(&final_dest)?.permissions();
                perm.set_mode(source_metadata.permissions().mode());
                fs::set_permissions(&final_dest, perm)?;
            }
            
            #[cfg(windows)]
            {
                let mut perm = fs::metadata(&final_dest)?.permissions();
                perm.set_readonly(source_metadata.permissions().readonly());
                fs::set_permissions(&final_dest, perm)?;
            }

            format!(
                "{} Copied file {} to {}",
                "✔".bright_green(),
                source_path.display().to_string().cyan(),
                final_dest.display().to_string().cyan()
            )
        }
    } else {
        fs::rename(&source_path, &final_dest)?;
        format!(
            "{} Moved {} to {}",
            "✔".bright_green(),
            source_path.display().to_string().cyan(),
            final_dest.display().to_string().cyan()
        )
    };

    let raw_path = final_dest.display().to_string();
    Ok((output, raw_path))
}

fn cmd_alias(alias_manager: &mut AliasManager, args: &[&str]) -> Result<(String, String), Box<dyn std::error::Error>> {
    if args.is_empty() {
        let mut output = format!("{}\n\n", "📛 Alias List:".bright_yellow().bold());
        let aliases = alias_manager.list();
        if aliases.is_empty() {
            output.push_str(&format!("  {}\n", "No aliases defined yet.".bright_black()));
        } else {
            for (alias, path) in aliases {
                output.push_str(&format!("  {} -> {}\n", format!("@{}", alias).cyan().bold(), path.bright_cyan()));
            }
        }
        output.push_str(&format!("\n{} Usage:\n", "💡".bright_green()));
        output.push_str(&format!("  {}    Add/Update alias\n", "alias add <name> <path>".cyan().bold()));
        output.push_str(&format!("  {}    Remove alias\n", "alias remove <name>".cyan().bold()));
        output.push_str(&format!("  {}         List all aliases\n", "alias list".cyan().bold()));
        return Ok((output, String::new()));
    }
    
    match args[0].to_lowercase().as_str() {
        "add" | "set" => {
            if args.len() < 3 {
                return Err("Usage: alias add <name> <path>".into());
            }
            let alias = args[1];
            let path = args[2];
            alias_manager.add(alias, path)?;
            Ok((format!("{} Added alias {} -> {}", "✔".bright_green(), format!("@{}", alias).cyan(), path.bright_cyan()), String::new()))
        }
        "remove" | "rm" | "delete" => {
            if args.len() < 2 {
                return Err("Usage: alias remove <name>".into());
            }
            let alias = args[1];
            alias_manager.remove(alias)?;
            Ok((format!("{} Removed alias {}", "✔".bright_green(), format!("@{}", alias).cyan()), String::new()))
        }
        "list" | "ls" => {
            let mut output = format!("{}\n\n", "📛 Alias List:".bright_yellow().bold());
            let aliases = alias_manager.list();
            if aliases.is_empty() {
                output.push_str(&format!("  {}\n", "No aliases defined yet.".bright_black()));
            } else {
                for (alias, path) in aliases {
                    output.push_str(&format!("  {} -> {}\n", format!("@{}", alias).cyan().bold(), path.bright_cyan()));
                }
            }
            Ok((output, String::new()))
        }
        _ => {
            Err(format!("Unknown alias subcommand: {}", args[0]).into())
        }
    }
}

fn cmd_tag(tag_manager: &mut TagManager, args: &[&str]) -> Result<(String, String), Box<dyn std::error::Error>> {
    if args.is_empty() {
        let mut output = format!("{}\n\n", "🏷️  标签管理命令帮助:".bright_yellow().bold());
        output.push_str(&format!("  {} <文件> <标签1> [标签2...] 添加标签到文件\n", "tag add".cyan().bold()));
        output.push_str(&format!("  {} <文件> <标签1> [标签2...] 移除文件的指定标签\n", "tag remove/rm".cyan().bold()));
        output.push_str(&format!("  {} <文件>                移除文件的所有标签\n", "tag clear".cyan().bold()));
        output.push_str(&format!("  {} <文件>                查看文件的所有标签\n", "tag get".cyan().bold()));
        output.push_str(&format!("  {}                        列出所有带标签的文件\n", "tag list/ls".cyan().bold()));
        output.push_str(&format!("  {} <标签正则1> [标签正则2...] 全局搜索所有匹配标签的文件\n", "tag find/search".cyan().bold()));
        output.push_str(&format!("  {}                        备份标签数据\n", "tag backup".cyan().bold()));
        output.push_str(&format!("  {}                        从备份恢复标签数据\n", "tag restore".cyan().bold()));
        return Ok((output, String::new()));
    }
    
    match args[0].to_lowercase().as_str() {
        "add" => {
            if args.len() < 3 {
                return Err("用法: tag add <文件> <标签1> [标签2...]".into());
            }
            let file_path = args[1];
            let tags = &args[2..];
            tag_manager.add_tags(file_path, tags)?;
            Ok((format!("✔️  已为文件 {} 添加标签: {}", file_path.cyan(), tags.join(", ").bright_yellow()), String::new()))
        }
        "remove" | "rm" => {
            if args.len() < 3 {
                return Err("用法: tag remove <文件> <标签1> [标签2...]".into());
            }
            let file_path = args[1];
            let tags = &args[2..];
            tag_manager.remove_tags(file_path, tags)?;
            Ok((format!("✔️  已为文件 {} 移除标签: {}", file_path.cyan(), tags.join(", ").bright_yellow()), String::new()))
        }
        "clear" => {
            if args.len() < 2 {
                return Err("用法: tag clear <文件>".into());
            }
            let file_path = args[1];
            tag_manager.remove_all_tags(file_path)?;
            Ok((format!("✔️  已为文件 {} 移除所有标签", file_path.cyan()), String::new()))
        }
        "get" => {
            if args.len() < 2 {
                return Err("用法: tag get <文件>".into());
            }
            let file_path = args[1];
            let tags = tag_manager.get_tags(file_path);
            if tags.is_empty() {
                Ok((format!("ℹ️  文件 {} 没有任何标签", file_path.cyan()), String::new()))
            } else {
                Ok((format!("🏷️  文件 {} 的标签: {}", file_path.cyan(), tags.join(", ").bright_yellow()), String::new()))
            }
        }
        "list" | "ls" => {
            let mut output = format!("{}\n\n", "🏷️  所有带标签的文件:".bright_yellow().bold());
            let all_tags = tag_manager.list_all();
            if all_tags.is_empty() {
                output.push_str(&format!("  {}\n", "没有任何标签记录".bright_black()));
            } else {
                for (path, tags) in all_tags {
                    // 处理Windows扩展路径前缀
                    let mut clean_path = path.clone();
                    if cfg!(windows) && clean_path.starts_with("\\\\?\\") {
                        clean_path = clean_path[4..].to_string();
                    }
                    
                    // 优先显示相对路径更友好
                    let display_path = match Path::new(&clean_path).strip_prefix(env::current_dir()?) {
                        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                        Err(_) => clean_path
                    };
                    output.push_str(&format!("  {} -> {}\n", display_path.cyan(), tags.join(", ").bright_yellow()));
                }
            }
            Ok((output, String::new()))
        }
        "backup" => {
            tag_manager.backup()?;
            Ok(("✔️  标签数据已备份成功".bright_green().to_string(), String::new()))
        }
        "find" | "search" => {
            if args.len() < 2 {
                return Err("用法: tag find <标签正则1> [标签正则2...]".into());
            }
            
            // 编译所有标签正则
            let mut tag_patterns = Vec::new();
            for pattern_str in &args[1..] {
                match Regex::new(pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("标签正则表达式无效: {}", e).into()),
                }
            }
            
            let results = tag_manager.find_files_by_tags(&tag_patterns);
            let mut output = format!("{} 匹配标签的文件共{}个:\n\n", "🔍".bright_yellow().bold(), results.len());
            
            if results.is_empty() {
                output.push_str(&format!("  {}\n", "没有找到匹配的文件".bright_black()));
            } else {
                // 转换为相对路径显示
                let current_dir = env::current_dir()?;
                for (path, tags) in results {
                    let display_path = match Path::new(&path).strip_prefix(&current_dir) {
                        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                        Err(_) => path
                    };
                    output.push_str(&format!("  {} -> {}\n", display_path.cyan(), tags.join(", ").bright_yellow()));
                }
            }
            
            Ok((output, String::new()))
        }
        "restore" => {
            tag_manager.restore()?;
            Ok(("✔️  标签数据已从备份恢复成功".bright_green().to_string(), String::new()))
        }
        _ => {
            Err(format!("未知的tag子命令: {}", args[0]).into())
        }
    }
}

fn cmd_help() -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut output = format!("{}\n\n", "📖 Available Commands:".bright_yellow().bold());
    output.push_str(&format!("  {}               List contents of current directory\n", "ls".cyan().bold()));
    output.push_str(&format!("  {}            List with detailed information\n", "ls -l".cyan().bold()));
    output.push_str(&format!("  {}             List including hidden files\n", "ls -a".cyan().bold()));
    output.push_str(&format!("  {}       List contents of specified directory\n", "ls <path>".cyan().bold()));
    output.push_str(&format!("  {}       List files with their tags\n", "ls -tag".cyan().bold()));
    output.push_str(&format!("  {}      List files matching specified tag regex, supports multi-tag combinations\n", "ls -t/--tag <tag-regex>".cyan().bold()));
    output.push_str(&format!("  {}       Search for files/directories using regex pattern\n", "ls --re <pattern>".cyan().bold()));
    output.push_str(&format!("  {}    Search recursively with regex\n", "ls --re-deep <pattern>".cyan().bold()));
    output.push_str(&format!("  {}  Case-insensitive regex search\n", "ls --re --xcaps <pattern>".cyan().bold()));
    output.push_str(&format!("  {}  Case-insensitive recursive regex search\n\n", "ls --re-deep --xcaps <pattern>".cyan().bold()));

    output.push_str(&format!("{}\n", "📝 Common Regex Syntax:".bright_yellow().bold()));
    output.push_str(&format!("  {}  Match any single character                e.g. ls --re fi.e  =>  file, fine\n", ".".bright_cyan()));
    output.push_str(&format!("  {}  Match previous char 0+ times              e.g. ls --re a*   =>  a, aa, aaa\n", "*".bright_cyan()));
    output.push_str(&format!("  {}  Match previous char 1+ times              e.g. ls --re a+   =>  a, aa, aaa\n", "+".bright_cyan()));
    output.push_str(&format!("  {}  Match previous char 0 or 1 time           e.g. ls --re colou?r  =>  color, colour\n", "?".bright_cyan()));
    output.push_str(&format!("  {}  Match start of string                     e.g. ls --re ^src  =>  files starting with src\n", "^".bright_cyan()));
    output.push_str(&format!("  {}  Match end of string                       e.g. ls --re \\.rs$  =>  all .rs files\n", "$".bright_cyan()));
    output.push_str(&format!("  {}  Match any char in set                     e.g. ls --re [Ff]ile  =>  File, file\n", "[abc]".bright_cyan()));
    output.push_str(&format!("  {}  Match any char NOT in set                 e.g. ls --re [^Ff]ile  =>  aile, bile...\n", "[^abc]".bright_cyan()));
    output.push_str(&format!("  {}  OR logic, match either expression         e.g. ls --re \\.rs$|\\.toml$  =>  .rs and .toml files\n", "|".bright_cyan()));
    output.push_str(&format!("  {}  Grouping for combining expressions         e.g. ls --re (src|target)\\/  =>  files under src or target\n\n", "()".bright_cyan()));
    
    output.push_str(&format!("  {}              Print current working directory\n", "pwd".cyan().bold()));
    output.push_str(&format!("  {}   Copy current directory path to clipboard\n", "cppwd".cyan().bold()));
    output.push_str(&format!("  {}  Copy file absolute path to clipboard\n\n", "cpf <file>".cyan().bold()));
    
    output.push_str(&format!("  {}            Change to home directory\n", "cd".cyan().bold()));
    output.push_str(&format!("  {}         Change to parent directory\n", "cd ..".cyan().bold()));
    output.push_str(&format!("  {}     Change to specified directory\n\n", "cd <path>".cyan().bold()));
    
    output.push_str(&format!("  {}         Open file with default application / Open directory in file explorer\n", "open <path>".cyan().bold()));
    
    output.push_str(&format!("  {}    Move file/folder to destination\n", "mv <source> <dest>".cyan().bold()));
    output.push_str(&format!("  {}    Copy file/folder to destination (preserves original)\n\n", "mv <source> <dest> --cp".cyan().bold()));
    
    output.push_str(&format!("  {}    Create a file (auto-creates parent directories)\n", "mkdf -f <path>".cyan().bold()));
    output.push_str(&format!("  {}      Create a directory\n", "mkdf -d <path>".cyan().bold()));
    output.push_str(&format!("  {}   Create a directory with parents\n", "mkdf -d -p <path>".cyan().bold()));
    output.push_str(&format!("  {}     Show mkdf command help\n\n", "mkdf -h/--help".cyan().bold()));
    output.push_str(&format!("  {}   Search recursively with regex\n", "ls --re-deep <pattern>".cyan().bold()));
    
    output.push_str(&format!("  {}             Exit the program\n", "exit".cyan().bold()));
    output.push_str(&format!("  {}            Clear the screen\n", "clear".cyan().bold()));
    output.push_str(&format!("  {}             Show this help\n", "help".cyan().bold()));
    output.push_str(&format!("  {}            Manage path aliases\n\n", "alias".cyan().bold()));

    output.push_str(&format!("{}\n", "⌨️ Keyboard Shortcuts:".bright_yellow().bold()));
    output.push_str(&format!("  {}        Clear current input line in REPL mode\n\n", "ESC".cyan().bold()));

    output.push_str(&format!("{}\n\n", "✨ Path Aliases:".bright_green().bold()));
    output.push_str(&format!("  Use {} prefix to use path aliases for faster navigation\n", "@".yellow().bold()));
    output.push_str("  Example:\n");
    output.push_str(&format!("    {}              Add alias for project directory\n", "alias add proj ~/projects".cyan()));
    output.push_str(&format!("    {}               List directory using alias\n", "ls @proj".cyan()));
    output.push_str(&format!("    {}        Navigate to subdirectory using alias\n", "cd @proj/rfe".cyan()));
    output.push_str(&format!("    {}                Open file using alias\n", "open @proj/rfe/src/main.rs".cyan()));
    output.push_str("  Aliases are saved persistently and available across sessions\n\n");

    output.push_str(&format!("{}\n\n", "✨ File Tags:".bright_green().bold()));
    output.push_str("  Add custom tags to files and directories for better organization\n");
    output.push_str("  Example:\n");
    output.push_str(&format!("    {}              Add tags 'work' and 'rust' to file\n", "tag add src/main.rs work rust".cyan()));
    output.push_str(&format!("    {}              Remove tag 'old' from file\n", "tag remove src/main.rs old".cyan()));
    output.push_str(&format!("    {}              Get all tags of file\n", "tag get src/main.rs".cyan()));
    output.push_str(&format!("    {}              List all files with tags\n", "tag list".cyan()));
    output.push_str(&format!("    {}              List files and their tags in current directory\n", "ls -tag".cyan()));
    output.push_str(&format!("    {}            List files in current directory tagged 'rust'\n", "ls -t rust".cyan()));
    output.push_str(&format!("    {}  Find files matching both 'rust' and 'doc' tags\n", "tag find rust doc".cyan()));
    output.push_str("  Supports regex matching, multi-tag queries, and automatic backup persistence\n\n");

    output.push_str(&format!("{}\n\n", "✨ Command Chain:".bright_green().bold()));
    output.push_str(&format!("  Use {} to chain commands with sequential execution and output passing\n", "->".yellow().bold()));
    output.push_str(&format!("  Example: {} pwd -> ls -> cd .. -> pwd\n", "$".bright_black()));
    output.push_str(&format!("  Use {} to continue execution even if previous command fails\n", "->!".yellow().bold()));
    output.push_str(&format!("  Example: {} cd non_exist! -> ls\n", "$".bright_black()));
    output.push_str(&format!("  Use {{}} as placeholder to insert previous command's output\n"));
    output.push_str(&format!("  Example: {} cppwd -> alias add desktop {{}}\n\n", "$".bright_black()));
    
    Ok((output, String::new()))
}

fn execute_single_command(input: &str, input_data: &str, alias_manager: &mut AliasManager, tag_manager: &mut TagManager) -> Result<(bool, String, String), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return Ok((false, String::new(), String::new()));
    }

    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "pwd" => {
            let (display, raw) = cmd_pwd()?;
            Ok((false, display, raw))
        }
        "cppwd" => {
            let (display, raw) = cmd_cppwd()?;
            Ok((false, display, raw))
        }
        "cpf" => {
            let path = if let Some(p) = parts.get(1).copied() {
                p.to_string()
            } else if !input_data.is_empty() {
                input_data.to_string()
            } else {
                return Err("Usage: cpf <file>".into());
            };
            let resolved_path = alias_manager.resolve_path(&path);
            let (display, raw) = cmd_cpf(&resolved_path)?;
            Ok((false, display, raw))
        }
        "cd" => {
            let path = if parts.len() > 1 {
                Some(alias_manager.resolve_path(parts[1]))
            } else if !input_data.is_empty() {
                Some(input_data.to_string())
            } else {
                None
            };
            let (display, raw) = cmd_cd(path.as_deref())?;
            Ok((false, display, raw))
        }
        "ls" => {
            let mut all = false;
            let mut long = false;
            let mut re = false;
            let mut re_insensitive = false;
            let mut show_tags = false;
            let mut recursive = false;
            let mut path: Option<String> = None;
            let mut tag_pattern_strs: Vec<String> = Vec::new();
            
            let mut i = 1;
            while i < parts.len() {
                match parts[i] {
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
                        if i + 1 < parts.len() {
                            tag_pattern_strs.push(parts[i+1].to_string());
                            i += 1;
                        } else {
                            return Err("标签查询参数需要指定匹配模式，用法：ls -t <标签正则>".into());
                        }
                    }
                    p => path = Some(alias_manager.resolve_path(p)),
                }
                i += 1;
            }
            
            // Use input data if no path provided
            if path.is_none() && !input_data.is_empty() {
                path = Some(input_data.to_string());
            }
            
            // 编译标签正则
            let mut tag_patterns = Vec::new();
            for pattern_str in tag_pattern_strs {
                match Regex::new(&pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("标签正则表达式无效: {}", e).into()),
                }
            }
            
            let (display, raw) = cmd_ls(all, long, re, re_insensitive, show_tags, recursive, path.as_deref(), tag_manager, &tag_patterns)?;
            Ok((false, display, raw))
        }
        "open" => {
            let path = if let Some(p) = parts.get(1).copied() {
                p.to_string()
            } else if !input_data.is_empty() {
                input_data.to_string()
            } else {
                return Err("Usage: open <file>".into());
            };
            let resolved_path = alias_manager.resolve_path(&path);
            let (display, raw) = cmd_open(&resolved_path)?;
            Ok((false, display, raw))
        }
        "mv" => {
            let mut source: Option<String> = None;
            let mut destination: Option<String> = None;
            let mut copy = false;

            for &part in &parts[1..] {
                if part == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(alias_manager.resolve_path(part));
                } else if destination.is_none() {
                    destination = Some(alias_manager.resolve_path(part));
                }
            }

            let source = source.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;
            let destination = destination.ok_or("Usage: mv <source_path> <destination_path> [--cp]")?;
            
            let (display, raw) = cmd_mv(&source, &destination, copy)?;
            Ok((false, display, raw))
        }
        "alias" => {
            let (display, raw) = cmd_alias(alias_manager, &parts[1..])?;
            Ok((false, display, raw))
        }
        "tag" | "t" => {
            let (display, raw) = cmd_tag(tag_manager, &parts[1..])?;
            Ok((false, display, raw))
        }
        "exit" | "quit" | "q" => {
            Ok((true, "👋 Goodbye!".bright_green().to_string(), String::new()))
        }
        "clear" | "cls" => {
            let (display, raw) = cmd_clear()?;
            Ok((false, display, raw))
        }
        "help" | "?" => {
            let (display, raw) = cmd_help()?;
            Ok((false, display, raw))
        }
        "mkdf" => {
            let (display, raw) = cmd_mkdf(&parts[1..])?;
            Ok((false, display, raw))
        }
        _ => {
            let display = format!(
                "{} Command not found: {}. Type 'help' for available commands.",
                "❌".red(),
                cmd.cyan()
            );
            Ok((false, display, String::new()))
        }
    }
}

fn execute_command(input: &str, alias_manager: &mut AliasManager, tag_manager: &mut TagManager) -> Result<bool, Box<dyn std::error::Error>> {
    let input = input.replace("\n", " ");
    let command_segments: Vec<&str> = input.split("->").map(|s| s.trim()).collect();
    
    let mut previous_raw_data = String::new();
    let mut should_exit = false;

    for segment in command_segments.iter() {
        if segment.is_empty() {
            continue;
        }

        let continue_on_error = segment.ends_with('!');
        let cmd = if continue_on_error { &segment[..segment.len()-1] } else { segment };
        
        // 使用 {} 占位符扩展：将前一个命令的原始输出替换到 {} 位置
        let cmd = if cmd.contains("{}") {
            cmd.replace("{}", &previous_raw_data)
        } else {
            cmd.to_string()
        };

        match execute_single_command(&cmd, &previous_raw_data, alias_manager, tag_manager) {
            Ok((exit, display_output, raw_data)) => {
                println!("{}", display_output);
                previous_raw_data = raw_data;
                if exit {
                    should_exit = true;
                    break;
                }
            }
            Err(e) => {
                let error_msg = format!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                println!("{}", error_msg);
                previous_raw_data = String::new();
                if !continue_on_error {
                    return Err(error_msg.into());
                }
            }
        }
    }

    Ok(should_exit)
}

fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    print_welcome();

    let mut alias_manager = AliasManager::new()?;
    let mut tag_manager = TagManager::new()?;
    
    let helper = RfeHelper {
        completer: FilenameCompleter::new(),
        alias_manager: AliasManager::new()?,
        tag_manager: TagManager::new()?,
    };

    let mut rl = rustyline::Editor::new()?;
    
    // 绑定ESC键清空当前输入行
    rl.bind_sequence(
        KeyEvent(KeyCode::Esc, rustyline::Modifiers::NONE),
        Cmd::Kill(Movement::WholeLine),
    );
    
    rl.set_helper(Some(helper));

    loop {
        let prompt = get_prompt_string();
        match rl.readline(&prompt) {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);

                match execute_command(input, &mut alias_manager, &mut tag_manager) {
                    Ok(should_exit) => {
                        if should_exit {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "👋 Goodbye!".bright_green());
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "👋 Goodbye!".bright_green());
                break;
            }
            Err(e) => {
                eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        if let Err(e) = run_repl() {
            eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
            std::process::exit(1);
        }
        return Ok(());
    }

    let mut alias_manager = AliasManager::new()?;
    let mut tag_manager = TagManager::new()?;
    let cmd = &args[1].to_lowercase();
    let result = match cmd.as_str() {
        "pwd" => cmd_pwd(),
        "cppwd" => cmd_cppwd(),
        "cpf" => {
            let path = args.get(2).map(|s| s.as_str()).ok_or("Usage: rfe cpf <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            cmd_cpf(&resolved_path)
        }
        "cd" => {
            let path = args.get(2).map(|s| s.as_str());
            let resolved_path = path.map(|p| alias_manager.resolve_path(p));
            cmd_cd(resolved_path.as_deref())
        }
        "ls" => {
            let mut all = false;
            let mut long = false;
            let mut re = false;
            let mut re_insensitive = false;
            let mut show_tags = false;
            let mut recursive = false;
            let mut path: Option<String> = None;
            let mut tag_pattern_strs: Vec<String> = Vec::new();
            
            let mut i = 2;
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
                            tag_pattern_strs.push(args[i+1].clone());
                            i += 1;
                        } else {
                            return Err("标签查询参数需要指定匹配模式，用法：ls -t <标签正则>".into());
                        }
                    }
                    p => path = Some(alias_manager.resolve_path(p)),
                }
                i += 1;
            }
            
            // 编译标签正则
            let mut tag_patterns = Vec::new();
            for pattern_str in tag_pattern_strs {
                match Regex::new(&pattern_str) {
                    Ok(re) => tag_patterns.push(re),
                    Err(e) => return Err(format!("标签正则表达式无效: {}", e).into()),
                }
            }
            
            cmd_ls(all, long, re, re_insensitive, show_tags, recursive, path.as_deref(), &tag_manager, &tag_patterns)
        }
        "open" => {
            let path = args.get(2).map(|s| s.as_str()).ok_or("Usage: rfe open <file>")?;
            let resolved_path = alias_manager.resolve_path(path);
            cmd_open(&resolved_path)
        }
        "mv" => {
            let mut source: Option<String> = None;
            let mut destination: Option<String> = None;
            let mut copy = false;

            for arg in &args[2..] {
                if arg == "--cp" {
                    copy = true;
                } else if source.is_none() {
                    source = Some(alias_manager.resolve_path(arg));
                } else if destination.is_none() {
                    destination = Some(alias_manager.resolve_path(arg));
                }
            }

            let source = source.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            let destination = destination.ok_or("Usage: rfe mv <source_path> <destination_path> [--cp]")?;
            
            cmd_mv(&source, &destination, copy)
        }
        "alias" => {
            let alias_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            cmd_alias(&mut alias_manager, &alias_args)
        }
        "tag" | "t" => {
            let tag_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            cmd_tag(&mut tag_manager, &tag_args)
        }
        "exit" => {
            Ok(("👋 Goodbye!".bright_green().to_string(), String::new()))
        }
        "clear" => cmd_clear(),
        "help" => cmd_help(),
        "mkdf" => {
            let mkdf_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            cmd_mkdf(&mkdf_args)
        }
        _ => {
            Ok((format!(
                "{} Command not found: {}. Type 'rfe help' for available commands.",
                "❌".red(),
                cmd.cyan()
            ), String::new()))
        }
    };

    match result {
        Ok((output, _raw)) => println!("{}", output),
        Err(e) => {
            eprintln!("{} {}", "❌ Error:".red(), e.to_string().bright_red());
            std::process::exit(1);
        }
    }

    Ok(())
}
