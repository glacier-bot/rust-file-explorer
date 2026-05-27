use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct AliasManager {
    pub aliases: HashMap<String, String>,
    pub config_path: PathBuf,
}

impl AliasManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.aliases)?;
        let mut file = File::create(&self.config_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
    
    pub fn add(&mut self, alias: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if alias.is_empty() {
            return Err("Alias cannot be empty".into());
        }
        if path.is_empty() {
            return Err("Path cannot be empty".into());
        }
        
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            return Err(format!("Path does not exist or is not accessible: {}", path).into());
        }
        
        self.aliases.insert(alias.to_string(), path.to_string());
        self.save()?;
        Ok(())
    }
    
    pub fn remove(&mut self, alias: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.aliases.remove(alias).is_none() {
            return Err(format!("Alias '{}' does not exist", alias).into());
        }
        self.save()?;
        Ok(())
    }
    
    pub fn get(&self, alias: &str) -> Option<&String> {
        self.aliases.get(alias)
    }
    
    pub fn list(&self) -> &HashMap<String, String> {
        &self.aliases
    }
    
    pub fn resolve_path(&self, path: &str) -> String {
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
    pub fn resolve_all_paths(&self, args: &[&str]) -> Vec<String> {
        args.iter().map(|&s| self.resolve_path(s)).collect()
    }
}