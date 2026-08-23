pub mod alias;
pub mod tag;

use std::path::PathBuf;

/// 返回 rfe 配置目录：优先 RFE_CONFIG_DIR 环境变量，否则为系统配置目录下的 rfe 子目录
pub fn config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(dir) = std::env::var("RFE_CONFIG_DIR") {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }
    Ok(dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("rfe"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::{tempdir, TempDir};

    /// 构造配置隔离的 TagManager（写入临时目录，不触碰真实用户配置）
    fn test_tag_manager(config: &TempDir) -> tag::TagManager {
        tag::TagManager::with_config_dir(config.path().to_path_buf()).unwrap()
    }

    /// 构造配置隔离的 AliasManager（写入临时目录，不触碰真实用户配置）
    fn test_alias_manager(config: &TempDir) -> alias::AliasManager {
        alias::AliasManager::with_config_dir(config.path().to_path_buf()).unwrap()
    }

    #[test]
    fn test_tag_manager_only_allow_files() {
        let config = tempdir().unwrap();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let mut tag_manager = test_tag_manager(&config);

        let result = tag_manager.add_tags(test_file.to_str().unwrap(), &["test_tag"]);
        assert!(result.is_ok(), "Should be able to add tags to files");

        let result = tag_manager.add_tags(temp_dir.path().to_str().unwrap(), &["test_tag"]);
        assert!(result.is_err(), "Should not be able to add tags to directories");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot add tags to directory"));
        assert!(error_msg.contains("Tags can only be added to files"));

        let result = tag_manager.remove_tags(test_file.to_str().unwrap(), &["test_tag"]);
        assert!(result.is_ok(), "Should be able to remove tags from files");

        let result = tag_manager.remove_tags(temp_dir.path().to_str().unwrap(), &["test_tag"]);
        assert!(result.is_err(), "Should not be able to remove tags from directories");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot remove tags from directory"));
        assert!(error_msg.contains("Tags can only be removed from files"));

        tag_manager.add_tags(test_file.to_str().unwrap(), &["another_tag"]).unwrap();
        let result = tag_manager.remove_all_tags(test_file.to_str().unwrap());
        assert!(result.is_ok(), "Should be able to remove all tags from files");

        let result = tag_manager.remove_all_tags(temp_dir.path().to_str().unwrap());
        assert!(result.is_err(), "Should not be able to remove all tags from directories");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Cannot remove tags from directory"));
        assert!(error_msg.contains("Tags can only be removed from files"));
    }

    #[test]
    fn test_alias_manager_only_allow_existing_paths() {
        let config = tempdir().unwrap();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let mut alias_manager = test_alias_manager(&config);

        let result = alias_manager.add("file_alias", test_file.to_str().unwrap());
        assert!(result.is_ok(), "Should be able to add alias to existing files");

        let result = alias_manager.add("dir_alias", temp_dir.path().to_str().unwrap());
        assert!(result.is_ok(), "Should be able to add alias to existing directories");

        let non_existent_path = temp_dir.path().join("non_existent_path");
        let result = alias_manager.add("bad_alias", non_existent_path.to_str().unwrap());
        assert!(result.is_err(), "Should not be able to add alias to non-existent paths");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Path does not exist or is not accessible"));

        let result = alias_manager.add("empty_path", "");
        assert!(result.is_err(), "Should not be able to add alias with empty path");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Path cannot be empty"));

        let result = alias_manager.add("", test_file.to_str().unwrap());
        assert!(result.is_err(), "Should not be able to add empty alias name");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Alias cannot be empty"));
    }

    #[test]
    fn test_alias_resolve_path() {
        let config = tempdir().unwrap();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let mut alias_manager = test_alias_manager(&config);
        alias_manager.add("myfile", test_file.to_str().unwrap()).unwrap();

        let resolved = alias_manager.resolve_path("@myfile");
        assert_eq!(resolved, test_file.to_str().unwrap());

        let resolved = alias_manager.resolve_path("@myfile/child.txt");
        assert_eq!(resolved, test_file.join("child.txt").display().to_string());

        let resolved = alias_manager.resolve_path(r"@myfile\child.txt");
        assert_eq!(resolved, test_file.join("child.txt").display().to_string());

        let resolved = alias_manager.resolve_path("@nonexistent");
        assert_eq!(resolved, "@nonexistent");

        let normal_path = "/some/normal/path";
        let resolved = alias_manager.resolve_path(normal_path);
        assert_eq!(resolved, normal_path);
    }

    #[test]
    fn test_tag_nonexistent_file() {
        let config = tempdir().unwrap();
        let mut tag_manager = test_tag_manager(&config);

        let result = tag_manager.add_tags("/nonexistent/file.txt", &["test_tag"]);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("File does not exist"));
    }

    #[test]
    fn test_tag_placeholder_index_roundtrip() {
        let config = tempdir().unwrap();
        let work = tempdir().unwrap();
        let folder1 = work.path().join("folder1");
        std::fs::create_dir(&folder1).unwrap();
        let placeholder = folder1.join(".index");
        let placeholder_str = placeholder.to_str().unwrap();

        let mut tag_manager = test_tag_manager(&config);

        // Given: folder1 下不存在 .index 文件
        // When: 以占位符添加标签
        tag_manager.add_tags(placeholder_str, &["newtag1"]).unwrap();
        // Then: 不产生实体文件
        assert!(!placeholder.exists(), ".index must remain a pure placeholder");
        // Then: 键以 folder1/.index 结尾且可读回
        let expected_suffix = format!("folder1{}.index", std::path::MAIN_SEPARATOR);
        assert!(tag_manager.list_all().keys().any(|k| k.ends_with(&expected_suffix)));
        assert_eq!(tag_manager.get_tags(placeholder_str), vec!["newtag1".to_string()]);

        // When/Then: remove 与 clear 同样适用于不存在的占位符
        tag_manager.remove_tags(placeholder_str, &["newtag1"]).unwrap();
        assert!(tag_manager.get_tags(placeholder_str).is_empty());
        tag_manager.add_tags(placeholder_str, &["t1", "t2"]).unwrap();
        tag_manager.remove_all_tags(placeholder_str).unwrap();
        assert!(tag_manager.get_tags(placeholder_str).is_empty());
    }

    #[test]
    fn test_tag_placeholder_key_lexical_normalization() {
        let config = tempdir().unwrap();
        let work = tempdir().unwrap();
        let folder1 = work.path().join("folder1");
        std::fs::create_dir(&folder1).unwrap();

        let mut tag_manager = test_tag_manager(&config);

        // When: 分别以 ./ 与 ../ 冗余写法指向同一占位符
        let via_dot = folder1.join(".").join(".index");
        let via_pop = folder1.join("sub").join("..").join(".index");
        tag_manager.add_tags(via_dot.to_str().unwrap(), &["tag_a"]).unwrap();
        tag_manager.add_tags(via_pop.to_str().unwrap(), &["tag_b"]).unwrap();

        // Then: 词法归一化后落在同一个键上
        assert_eq!(tag_manager.list_all().len(), 1);
        let tags = tag_manager.get_tags(folder1.join(".index").to_str().unwrap());
        assert!(tags.contains(&"tag_a".to_string()));
        assert!(tags.contains(&"tag_b".to_string()));
    }

    #[test]
    fn test_tag_existing_index_file_uses_canonical_key() {
        let config = tempdir().unwrap();
        let work = tempdir().unwrap();
        let folder1 = work.path().join("folder1");
        std::fs::create_dir(&folder1).unwrap();
        let index_file = folder1.join(".index");
        File::create(&index_file).unwrap();

        let mut tag_manager = test_tag_manager(&config);
        tag_manager.add_tags(index_file.to_str().unwrap(), &["realfile"]).unwrap();

        // Then: 已存在的 .index 仍走 canonicalize（键不带 \\?\ 前缀）
        let key = tag_manager.list_all().keys().next().unwrap();
        let expected_suffix = format!("folder1{}.index", std::path::MAIN_SEPARATOR);
        assert!(key.ends_with(&expected_suffix));
        assert!(!key.starts_with("\\\\?\\"));
    }

    #[test]
    fn test_config_dir_env_override_and_fallback() {
        // Given: 设置 RFE_CONFIG_DIR 覆盖
        let override_dir = tempdir().unwrap();
        std::env::set_var("RFE_CONFIG_DIR", override_dir.path());
        // Then: 优先使用覆盖目录
        assert_eq!(config_dir().unwrap(), override_dir.path());

        // When: 移除覆盖
        std::env::remove_var("RFE_CONFIG_DIR");
        // Then: 回退到系统配置目录下的 rfe 子目录
        assert_eq!(config_dir().unwrap().file_name().unwrap(), "rfe");
    }

    #[test]
    fn test_managers_persist_into_injected_config_dir() {
        let config = tempdir().unwrap();
        let work = tempdir().unwrap();
        let test_file = work.path().join("a.txt");
        File::create(&test_file).unwrap();

        let mut tag_manager = test_tag_manager(&config);
        tag_manager.add_tags(test_file.to_str().unwrap(), &["x"]).unwrap();
        let mut alias_manager = test_alias_manager(&config);
        alias_manager.add("wrk", work.path().to_str().unwrap()).unwrap();

        // Then: 数据写入注入目录
        assert!(config.path().join("tags.json").exists());
        assert!(config.path().join("aliases.json").exists());

        // Then: 重新构造后可加载（持久化生效）
        let tag_manager2 = test_tag_manager(&config);
        assert_eq!(tag_manager2.get_tags(test_file.to_str().unwrap()), vec!["x".to_string()]);
        let alias_manager2 = test_alias_manager(&config);
        assert_eq!(alias_manager2.get("wrk").unwrap(), work.path().to_str().unwrap());
    }
}
