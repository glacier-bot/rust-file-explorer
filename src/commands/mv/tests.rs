use super::*;
use crate::utils::moe::{disable_moe, enable_moe};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_mv_file_to_nonexistent_dir_returns_error() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("test_file.txt");
    fs::write(&source_path, "test content").unwrap();

    let result = cmd_mv(source_path.to_str().unwrap(), "nonexistent_dir/", false);

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Destination directory does not exist"));
    assert!(source_path.exists());
}

#[test]
fn test_mv_file_with_nonexistent_parent_dir_returns_error() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("test_file.txt");
    fs::write(&source_path, "test content").unwrap();

    let result = cmd_mv(
        source_path.to_str().unwrap(),
        "nonexistent_parent/new_file.txt",
        false,
    );

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Destination parent directory does not exist"));
    assert!(source_path.exists());
}

#[test]
fn test_mv_file_to_existing_dir() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("test_file.txt");
    let dest_dir = temp_dir.path().join("existing_dir");
    fs::write(&source_path, "test content").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    let result = cmd_mv(
        source_path.to_str().unwrap(),
        dest_dir.to_str().unwrap(),
        false,
    );

    assert!(result.is_ok());
    let (output, raw) = result.unwrap();
    assert!(output.contains("Moved"));
    assert!(raw.contains("existing_dir") && raw.contains("test_file.txt"));
    assert!(!source_path.exists());
    assert!(dest_dir.join("test_file.txt").exists());
}

#[test]
fn test_mv_rename_file() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("old_name.txt");
    let dest_path = temp_dir.path().join("new_name.txt");
    fs::write(&source_path, "test content").unwrap();

    let result = cmd_mv(
        source_path.to_str().unwrap(),
        dest_path.to_str().unwrap(),
        false,
    );

    assert!(result.is_ok());
    let (output, raw) = result.unwrap();
    assert!(output.contains("Moved"));
    assert!(raw.contains("new_name.txt"));
    assert!(!source_path.exists());
    assert!(dest_path.exists());
}

#[test]
fn test_mv_destination_already_exists_returns_error() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("file_a.txt");
    let dest_path = temp_dir.path().join("file_b.txt");
    fs::write(&source_path, "content a").unwrap();
    fs::write(&dest_path, "content b").unwrap();

    let result = cmd_mv(
        source_path.to_str().unwrap(),
        dest_path.to_str().unwrap(),
        false,
    );

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Destination path already exists"));
    assert!(source_path.exists());
    assert!(dest_path.exists());
}

#[test]
fn test_mv_source_not_exists_returns_error() {
    let result = cmd_mv("nonexistent_source.txt", "some_dir/", false);

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Source path does not exist"));
}

#[test]
fn test_mv_copy_mode_to_nonexistent_dir_returns_error() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("test_file.txt");
    fs::write(&source_path, "test content").unwrap();

    let result = cmd_mv(source_path.to_str().unwrap(), "nonexistent_dir/", true);

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Destination directory does not exist"));
    assert!(source_path.exists());
}

#[test]
fn test_mv_copy_mode_to_existing_dir() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("test_file.txt");
    let dest_dir = temp_dir.path().join("existing_dir");
    fs::write(&source_path, "test content").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    let result = cmd_mv(
        source_path.to_str().unwrap(),
        dest_dir.to_str().unwrap(),
        true,
    );

    assert!(result.is_ok());
    let (output, _raw) = result.unwrap();
    assert!(output.contains("Copied"));
    assert!(source_path.exists());
    assert!(dest_dir.join("test_file.txt").exists());
}

#[test]
fn test_mv_error_message_std_mode() {
    disable_moe();
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("test_file.txt");
    fs::write(&source_path, "test content").unwrap();

    let result = cmd_mv(source_path.to_str().unwrap(), "nonexistent_dir/", false);

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("Error:"));
    assert!(err_msg.contains("Destination directory does not exist"));
}

#[test]
fn test_mv_error_message_moe_mode() {
    enable_moe();
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("test_file.txt");
    fs::write(&source_path, "test content").unwrap();

    let result = cmd_mv(source_path.to_str().unwrap(), "nonexistent_dir/", false);

    assert!(result.is_err());
    let err_msg = result.err().unwrap().to_string();
    assert!(err_msg.contains("😢"));
    assert!(err_msg.contains("💔"));
    assert!(err_msg.contains("Error:"));
    assert!(err_msg.contains("Destination directory does not exist"));
    disable_moe();
}

#[test]
fn test_mv_directory_to_existing_dir() {
    let temp_dir = tempdir().unwrap();
    let source_dir = temp_dir.path().join("source_dir");
    let dest_dir = temp_dir.path().join("dest_dir");
    fs::create_dir(&source_dir).unwrap();
    fs::write(source_dir.join("file.txt"), "content").unwrap();
    fs::create_dir(&dest_dir).unwrap();

    let result = cmd_mv(
        source_dir.to_str().unwrap(),
        dest_dir.to_str().unwrap(),
        false,
    );

    assert!(result.is_ok());
    let (output, _raw) = result.unwrap();
    assert!(output.contains("Moved"));
    assert!(!source_dir.exists());
    assert!(dest_dir.join("source_dir").exists());
    assert!(dest_dir.join("source_dir").join("file.txt").exists());
}
