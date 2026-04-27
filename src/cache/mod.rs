//! 目录缓存模块
//! 用于缓存目录内容，提高补全性能

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

/// 目录缓存结构体
/// 存储目录条目和缓存时间戳
pub struct DirCache {
    /// 目录条目列表，每个条目包含文件名和是否为目录
    pub entries: Vec<(String, bool)>,
    /// 缓存创建时间戳
    pub timestamp: Instant,
}

/// 全局目录缓存
pub static DIR_CACHE: Mutex<Option<HashMap<String, DirCache>>> = Mutex::new(None);

/// 缓存有效期（毫秒）
pub const CACHE_TTL_MS: u64 = 5000;

/// 获取缓存的目录条目
/// 如果缓存有效，返回缓存的条目；否则返回 None
pub fn get_cached_dir_entries(path: &Path) -> Option<Vec<(String, bool)>> {
    let path_str = path.to_string_lossy().to_string();
    
    let mut cache_guard = DIR_CACHE.lock().ok()?;
    
    if cache_guard.is_none() {
        *cache_guard = Some(HashMap::new());
    }
    
    let cache = cache_guard.as_ref().unwrap();
    
    if let Some(cached) = cache.get(&path_str) {
        if cached.timestamp.elapsed().as_millis() < CACHE_TTL_MS as u128 {
            return Some(cached.entries.clone());
        }
    }
    
    None
}

/// 缓存目录条目
pub fn cache_dir_entries(path: &Path, entries: Vec<(String, bool)>) {
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