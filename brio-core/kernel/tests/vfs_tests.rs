//! Extended tests for the VFS (Virtual File System) module.

use brio_kernel::vfs::manager::SessionManager;
use std::fs;

// =============================================================================
// Session Manager Tests
// =============================================================================

#[test]
fn test_session_begin_with_nonexistent_path() -> anyhow::Result<()> {
    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let result = manager.begin_session("/nonexistent/path/that/does/not/exist".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid base path"));
    Ok(())
}

#[test]
fn test_commit_nonexistent_session() -> anyhow::Result<()> {
    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let result = manager.commit_session("fake-session-id-12345".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
    Ok(())
}

#[test]
fn test_session_with_empty_directory() -> anyhow::Result<()> {
    let temp = std::env::temp_dir().join("brio_vfs_test_empty");
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }
    fs::create_dir_all(&temp)?;

    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let session_id = manager
        .begin_session(
            temp.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid temp dir"))?
                .to_string(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    // Session should be created even for empty directory
    let session_path = std::env::temp_dir().join("brio").join(&session_id);
    assert!(session_path.exists());

    // Commit should succeed with no changes
    let result = manager.commit_session(session_id);
    assert!(result.is_ok());

    // Cleanup
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[test]
fn test_session_preserves_nested_structure() -> anyhow::Result<()> {
    let temp = std::env::temp_dir().join("brio_vfs_test_nested");
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }

    // Create nested directory structure
    fs::create_dir_all(temp.join("a/b/c"))?;
    fs::write(temp.join("root.txt"), "root")?;
    fs::write(temp.join("a/level1.txt"), "level1")?;
    fs::write(temp.join("a/b/level2.txt"), "level2")?;
    fs::write(temp.join("a/b/c/level3.txt"), "level3")?;

    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let session_id = manager
        .begin_session(
            temp.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid temp dir"))?
                .to_string(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    let session_path = std::env::temp_dir().join("brio").join(&session_id);

    // Verify all files exist in session
    assert!(session_path.join("root.txt").exists());
    assert!(session_path.join("a/level1.txt").exists());
    assert!(session_path.join("a/b/level2.txt").exists());
    assert!(session_path.join("a/b/c/level3.txt").exists());

    // Verify content
    assert_eq!(
        fs::read_to_string(session_path.join("a/b/c/level3.txt"))?,
        "level3"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp);
    let _ = fs::remove_dir_all(&session_path);
    Ok(())
}

#[test]
fn test_session_modification_and_commit() -> anyhow::Result<()> {
    let temp = std::env::temp_dir().join("brio_vfs_test_modify");
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }
    fs::create_dir_all(&temp)?;
    fs::write(temp.join("file.txt"), "original")?;

    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let session_id = manager
        .begin_session(
            temp.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid temp dir"))?
                .to_string(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    let session_path = std::env::temp_dir().join("brio").join(&session_id);

    // Modify in session
    fs::write(session_path.join("file.txt"), "modified")?;

    // Commit
    manager
        .commit_session(session_id)
        .map_err(|e| anyhow::anyhow!(e))?;

    // Base should have the modified content
    assert_eq!(fs::read_to_string(temp.join("file.txt"))?, "modified");

    // Cleanup
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[test]
fn test_session_add_new_file() -> anyhow::Result<()> {
    let temp = std::env::temp_dir().join("brio_vfs_test_add");
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }
    fs::create_dir_all(&temp)?;

    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let session_id = manager
        .begin_session(
            temp.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid temp dir"))?
                .to_string(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    let session_path = std::env::temp_dir().join("brio").join(&session_id);

    // Add new file in session
    fs::write(session_path.join("new_file.txt"), "new content")?;

    // Commit
    manager
        .commit_session(session_id)
        .map_err(|e| anyhow::anyhow!(e))?;

    // New file should exist in base
    assert!(temp.join("new_file.txt").exists());
    assert_eq!(
        fs::read_to_string(temp.join("new_file.txt"))?,
        "new content"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

#[test]
fn test_session_delete_file() -> anyhow::Result<()> {
    let temp = std::env::temp_dir().join("brio_vfs_test_delete");
    if temp.exists() {
        fs::remove_dir_all(&temp)?;
    }
    fs::create_dir_all(&temp)?;
    fs::write(temp.join("to_delete.txt"), "delete me")?;

    let mut manager = SessionManager::new(Default::default()).map_err(|e| anyhow::anyhow!(e))?;
    let session_id = manager
        .begin_session(
            temp.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid temp dir"))?
                .to_string(),
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    let session_path = std::env::temp_dir().join("brio").join(&session_id);

    // Delete file in session
    fs::remove_file(session_path.join("to_delete.txt"))?;

    // Commit
    manager
        .commit_session(session_id)
        .map_err(|e| anyhow::anyhow!(e))?;

    // File should be deleted from base
    assert!(!temp.join("to_delete.txt").exists());

    // Cleanup
    let _ = fs::remove_dir_all(&temp);
    Ok(())
}

// =============================================================================
// SessionManager Default Trait Test
// =============================================================================

#[test]
fn test_session_manager_default() {
    let manager = SessionManager::default();
    // Just verify it can be created via Default trait
    drop(manager);
}
