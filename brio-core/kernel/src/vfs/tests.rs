use super::manager::SessionManager;
use crate::infrastructure::config::SandboxSettings;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_session_lifecycle() {
    // Setup base directory
    let temp_dir = std::env::temp_dir().join("brio_tests");
    let base_dir = temp_dir.join("base");

    if base_dir.exists() {
        fs::remove_dir_all(&base_dir).unwrap();
    }
    fs::create_dir_all(&base_dir).unwrap();

    // Create initial files
    fs::write(base_dir.join("file1.txt"), "original").unwrap();
    fs::create_dir(base_dir.join("subdir")).unwrap();
    fs::write(base_dir.join("subdir/file2.txt"), "sub").unwrap();

    // 1. Begin Session
    let mut manager = SessionManager::new(Default::default());
    let session_id = manager
        .begin_session(base_dir.to_str().unwrap().to_string())
        .unwrap();

    // Internal check
    let session_path = std::env::temp_dir().join("brio").join(&session_id);
    assert!(session_path.exists());
    assert_eq!(
        fs::read_to_string(session_path.join("file1.txt")).unwrap(),
        "original"
    );

    // 2. Modify Session
    fs::write(session_path.join("file1.txt"), "modified").unwrap();
    fs::write(session_path.join("new.txt"), "created").unwrap();
    fs::remove_file(session_path.join("subdir/file2.txt")).unwrap();

    // 3. Commit Session
    manager.commit_session(session_id).unwrap();

    // 4. Verify Base
    assert_eq!(
        fs::read_to_string(base_dir.join("file1.txt")).unwrap(),
        "modified"
    );
    assert_eq!(
        fs::read_to_string(base_dir.join("new.txt")).unwrap(),
        "created"
    );
    // Verify deletion
    assert!(!base_dir.join("subdir/file2.txt").exists());

    // Cleanup
    let _ = fs::remove_dir_all(&base_dir);
    let _ = fs::remove_dir_all(&session_path);
}

#[test]
fn test_begin_session_sandbox_violation() {
    // Setup a temp dir as our "allowed" root (though we won't put the target there)
    let temp_dir = tempdir().unwrap();
    let allowed_path = temp_dir.path().join("allowed_project");
    fs::create_dir(&allowed_path).unwrap();

    // Setup a target outside the allowed root
    let diff_path = temp_dir.path().join("forbidden_project");
    fs::create_dir(&diff_path).unwrap();

    let sandbox = SandboxSettings {
        allowed_paths: vec![allowed_path.to_string_lossy().to_string()],
    };
    let mut manager = SessionManager::new(sandbox);

    // Attempt to start session on forbidden path
    let result = manager.begin_session(diff_path.to_string_lossy().to_string());

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Security Violation"));
}

#[test]
fn test_begin_session_sandbox_ok() {
    let temp_dir = tempdir().unwrap();
    let allowed_path = temp_dir.path().join("allowed_project");
    fs::create_dir(&allowed_path).unwrap();

    let sandbox = SandboxSettings {
        allowed_paths: vec![allowed_path.to_string_lossy().to_string()],
    };
    let mut manager = SessionManager::new(sandbox);

    let result = manager.begin_session(allowed_path.to_string_lossy().to_string());
    assert!(result.is_ok());
}
