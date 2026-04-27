use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct TagManager {
    pub tags: HashMap<String, Vec<String>>,
    pub config_path: PathBuf,
    pub backup_path: PathBuf,
}

impl TagManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
                    if backup_path.exists() {
                        match Self::load_from_file(&backup_path) {
                            Ok(backup_tags) => {
                                tags = backup_tags;
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
            if let Ok(backup_tags) = Self::load_from_file(&backup_path) {
                tags = backup_tags;
                fs::copy(&backup_path, &config_path)?;
                eprintln!("⚠️  标签主文件丢失，已从备份恢复");
            }
        }
        
        Ok(Self { tags, config_path, backup_path })
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
            Err("标签文件不存在，无法备份".into())
        }
    }
    
    pub fn restore(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.backup_path.exists() {
            self.tags = Self::load_from_file(&self.backup_path)?;
            self.save()?;
            Ok(())
        } else {
            Err("备份文件不存在，无法恢复".into())
        }
    }
    
    pub fn add_tags(&mut self, file_path: &str, tags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(file_path);
        if !path.exists() {
            return Err(format!("文件或文件夹不存在: {}", file_path).into());
        }
        
        let abs_path = fs::canonicalize(&path)?
            .to_string_lossy()
            .to_string();
        
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
    
    pub fn remove_tags(&mut self, file_path: &str, tags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
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
    
    pub fn remove_all_tags(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    
    pub fn get_tags(&self, file_path: &str) -> Vec<String> {
        match fs::canonicalize(file_path) {
            Ok(abs_path) => {
                let path_str = abs_path.to_string_lossy().to_string();
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
        match fs::canonicalize(file_path) {
            Ok(abs_path) => {
                let path_str = abs_path.to_string_lossy().to_string();
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