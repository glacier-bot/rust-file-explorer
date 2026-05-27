pub mod alias;
pub mod tag;

#[cfg(test)]
mod tests { 
   use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_tag_manager_only_allow_files() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();
        
        let mut tag_manager = tag::TagManager::new().unwrap();
        
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
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();
        
        let mut alias_manager = alias::AliasManager::new().unwrap();
        
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
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();
        
        let mut alias_manager = alias::AliasManager::new().unwrap();
        alias_manager.add("myfile", test_file.to_str().unwrap()).unwrap();
        
        let resolved = alias_manager.resolve_path("@myfile");
        assert_eq!(resolved, test_file.to_str().unwrap());
        
        let resolved = alias_manager.resolve_path("@nonexistent");
        assert_eq!(resolved, "@nonexistent");
        
        let normal_path = "/some/normal/path";
        let resolved = alias_manager.resolve_path(normal_path);
        assert_eq!(resolved, normal_path);
    }

    #[test]
    fn test_tag_nonexistent_file() {
        let mut tag_manager = tag::TagManager::new().unwrap();
        
        let result = tag_manager.add_tags("/nonexistent/file.txt", &["test_tag"]);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("File does not exist"));
    }
}
