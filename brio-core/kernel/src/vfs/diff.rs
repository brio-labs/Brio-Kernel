use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum FileChange {
    Modified(PathBuf), // Path relative to base
    Added(PathBuf),
    Deleted(PathBuf),
}

/// Computes SHA256 hash of a file
fn compute_hash(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // 8KB buffer

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Scans a directory and returns a map of RelativePath -> Hash
fn scan_directory(root: &Path) -> io::Result<HashMap<PathBuf, String>> {
    let mut map = HashMap::new();

    for entry in WalkDir::new(root) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.strip_prefix(root).is_ok() {
            let relative = path.strip_prefix(root).unwrap();
            let hash = compute_hash(path)?;
            map.insert(relative.to_path_buf(), hash);
        }
    }

    Ok(map)
}

/// Compares session directory against base directory to find changes
pub fn compute_diff(session_path: &Path, base_path: &Path) -> io::Result<Vec<FileChange>> {
    let session_files = scan_directory(session_path)?;
    let base_files = scan_directory(base_path)?;

    let mut changes = Vec::new();

    // Check for Modified and Added
    for (rel_path, session_hash) in &session_files {
        match base_files.get(rel_path) {
            Some(base_hash) => {
                if session_hash != base_hash {
                    changes.push(FileChange::Modified(rel_path.clone()));
                }
            }
            None => {
                changes.push(FileChange::Added(rel_path.clone()));
            }
        }
    }

    // Check for Deleted
    for rel_path in base_files.keys() {
        if !session_files.contains_key(rel_path) {
            changes.push(FileChange::Deleted(rel_path.clone()));
        }
    }

    Ok(changes)
}

/// Applies changes from session to base
pub fn apply_changes(
    session_path: &Path,
    base_path: &Path,
    changes: &[FileChange],
) -> io::Result<()> {
    for change in changes {
        match change {
            FileChange::Added(rel) | FileChange::Modified(rel) => {
                let src = session_path.join(rel);
                let dst = base_path.join(rel);

                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Safest is copy.
                fs::copy(&src, &dst)?;
                debug!("Applied {:?}: {:?}", change, dst);
            }
            FileChange::Deleted(rel) => {
                let target = base_path.join(rel);
                if target.exists() {
                    fs::remove_file(&target)?;
                    debug!("Applied Delete: {:?}", target);
                }
            }
        }
    }

    info!("Applied {} changes to Base", changes.len());
    Ok(())
}
