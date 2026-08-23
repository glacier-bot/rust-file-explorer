use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

mod paths;

pub struct TagManager {
    pub tags: HashMap<String, Vec<String>>,
    pub config_path: PathBuf,
    pub backup_path: PathBuf,
}

impl TagManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config_dir(super::config_dir()?)
    }

    /// 使用指定配置目录构造（便于测试隔离）
    #[allow(dead_code)]
    pub fn with_config_dir(config_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("tags.json");
        let backup_path = config_dir.join("tags.json.bak");

        let mut tags = HashMap::new();

        if config_path.exists() {
            match Self::load_from_file(&config_path) {
                Ok(loaded_tags) => tags = loaded_tags,
                Err(e) => {
                    if backup_path.exists() {
                        match Self::load_from_file(&backup_path) {
                            Ok(backup_tags) => {
                                tags = backup_tags;
                                fs::copy(&backup_path, &config_path)?;
                                eprintln!("⚠️  Tag file corrupted, restored from backup.");
                            }
                            Err(_) => return Err(format!("Tag file corrupted and backup cannot be read: {}", e).into())
                        }
                    } else {
                        return Err(format!("Tag file corrupted and no backup available: {}", e).into())
                    }
                }
            }
        } else if backup_path.exists() {
            if let Ok(backup_tags) = Self::load_from_file(&backup_path) {
                tags = backup_tags;
                fs::copy(&backup_path, &config_path)?;
                eprintln!("⚠️  Tag file missing, restored from backup.");
            }
        }

        let need_save = paths::migrate_unc_paths(&mut tags);

        let manager = Self { tags, config_path, backup_path };

        if need_save {
            let _ = manager.save();
        }

        Ok(manager)
    }

    pub fn load_from_file(path: &PathBuf) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let temp_path = self.config_path.with_extension("tmp");
        let content = serde_json::to_string_pretty(&self.tags)?;

        let mut file = File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        if self.config_path.exists() {
            fs::copy(&self.config_path, &self.backup_path)?;
        }

        fs::rename(&temp_path, &self.config_path)?;

        Ok(())
    }

    pub fn backup(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config_path.exists() {
            fs::copy(&self.config_path, &self.backup_path)?;
            Ok(())
        } else {
            Err("Tag file does not exist, cannot backup.".into())
        }
    }

    pub fn restore(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.backup_path.exists() {
            self.tags = Self::load_from_file(&self.backup_path)?;
            self.save()?;
            Ok(())
        } else {
            Err("Backup file does not exist, cannot restore.".into())
        }
    }

    pub fn add_tags(&mut self, file_path: &str, tags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(file_path);
        let is_placeholder = paths::is_index_placeholder(&path);
        if !is_placeholder && !path.exists() {
            return Err(format!("File does not exist: {}", file_path).into());
        }

        if !is_placeholder && path.is_dir() {
            return Err(format!("Cannot add tags to directory: {}. Tags can only be added to files.", file_path).into());
        }

        let abs_path = paths::normalize_path(file_path)?;

        for tag in tags {
                if tag.is_empty() {
                    return Err("Tag cannot be empty.".into());
                }
                if tag.contains(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '/' || c == '\\') {
                    return Err(format!("Tag contains invalid characters: {}", tag).into());
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

    pub fn remove_tags(&mut self, file_path: &str, tags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(file_path);
        let is_placeholder = paths::is_index_placeholder(&path);
        if !is_placeholder && !path.exists() {
            return Err(format!("File does not exist: {}", file_path).into());
        }

        if !is_placeholder && path.is_dir() {
            return Err(format!("Cannot remove tags from directory: {}. Tags can only be removed from files.", file_path).into());
        }

        let abs_path = paths::normalize_path(file_path)?;

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
            Err(format!("No tags found for this file: {}", file_path).into())
        }
    }

    pub fn remove_all_tags(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(file_path);
        let is_placeholder = paths::is_index_placeholder(&path);
        if !is_placeholder && !path.exists() {
            return Err(format!("File does not exist: {}", file_path).into());
        }

        if !is_placeholder && path.is_dir() {
            return Err(format!("Cannot remove tags from directory: {}. Tags can only be removed from files.", file_path).into());
        }

        let abs_path = paths::normalize_path(file_path)?;

        if self.tags.remove(&abs_path).is_some() {
            self.save()?;
            Ok(())
        } else {
            Err(format!("No tags found for this file: {}", file_path).into())
        }
    }

    pub fn get_tags(&self, file_path: &str) -> Vec<String> {
        match paths::normalize_path(file_path) {
            Ok(path_str) => {
                self.tags.get(&path_str).cloned().unwrap_or_default()
            }
            Err(_) => Vec::new()
        }
    }

    pub fn get_all_tags(&self) -> Vec<String> {
        let mut all_tags = HashSet::new();
        for tags in self.tags.values() {
            for tag in tags {
                all_tags.insert(tag.clone());
            }
        }
        all_tags.into_iter().collect()
    }

    pub fn list_all(&self) -> &HashMap<String, Vec<String>> {
        &self.tags
    }

    pub fn file_matches_tags(&self, file_path: &str, tag_patterns: &[Regex]) -> bool {
        match paths::normalize_path(file_path) {
            Ok(path_str) => {
                match self.tags.get(&path_str) {
                    Some(tags) => {
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

    pub fn find_files_by_tags(&self, tag_patterns: &[Regex]) -> Vec<(String, Vec<String>)> {
        let mut result = Vec::new();
        for (path, tags) in &self.tags {
            if tag_patterns.iter().all(|pattern| {
                tags.iter().any(|tag| pattern.is_match(tag))
            }) {
                result.push((path.clone(), tags.clone()));
            }
        }
        result
    }
}
