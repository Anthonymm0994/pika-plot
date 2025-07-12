//! Windows-specific file system utilities.
//! Based on Gemini 2.5 Pro's recommendations for handling Windows quirks.

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{self, ErrorKind};
use std::time::Duration;
use anyhow::{Result, Context};

/// Normalize a Windows path, handling long paths and UNC paths.
/// Uses the `dunce` crate as recommended by Gemini.
pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    // dunce::canonicalize handles \\?\ prefix automatically
    match dunce::canonicalize(&path) {
        Ok(canonical) => canonical,
        Err(_) => {
            // If canonicalize fails (e.g., path doesn't exist yet),
            // just convert to PathBuf and clean it up
            let path = path.as_ref();
            
            // Handle relative paths
            if path.is_relative() {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(path)
            } else {
                path.to_path_buf()
            }
        }
    }
}

/// Check if a file is locked by another process.
pub fn is_file_locked(path: impl AsRef<Path>) -> bool {
    match File::open(&path) {
        Ok(_) => false,
        Err(e) => matches!(e.kind(), ErrorKind::PermissionDenied),
    }
}

/// Safely open a file with retry logic for locked files.
pub fn safe_open_file(
    path: impl AsRef<Path>,
    retries: u32,
    retry_delay: Duration,
) -> Result<File> {
    let path = path.as_ref();
    let mut last_error = None;
    
    for attempt in 0..=retries {
        match File::open(path) {
            Ok(file) => return Ok(file),
            Err(e) => {
                if attempt < retries {
                    match e.kind() {
                        ErrorKind::PermissionDenied => {
                            // Likely locked by another program (e.g., Excel)
                            last_error = Some(e);
                            std::thread::sleep(retry_delay);
                        }
                        _ => {
                            last_error = Some(e);
                            break; // Don't retry for other errors
                        }
                    }
                } else {
                    last_error = Some(e);
                }
            }
        }
    }
    
    let err = last_error.unwrap();
    if err.kind() == ErrorKind::PermissionDenied {
        Err(anyhow::anyhow!(
            "File '{}' is locked by another program (possibly Excel). Please close it and try again.",
            path.display()
        ))
    } else {
        Err(err).context(format!("Failed to open file '{}'", path.display()))
    }
}

/// Open a file for writing with Windows-specific options.
pub fn safe_create_file(path: impl AsRef<Path>) -> Result<File> {
    let path = normalize_path(path);
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create parent directories")?;
    }
    
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .context(format!("Failed to create file '{}'", path.display()))
}

/// Get user-friendly error message for file operations.
pub fn file_error_message(error: &io::Error, path: &Path) -> String {
    match error.kind() {
        ErrorKind::NotFound => format!("File not found: '{}'", path.display()),
        ErrorKind::PermissionDenied => {
            format!("Access denied: '{}'. The file may be open in another program.", path.display())
        }
        ErrorKind::AlreadyExists => format!("File already exists: '{}'", path.display()),
        _ => format!("File operation failed: '{}' - {}", path.display(), error),
    }
}

/// Handle case-insensitive path lookups.
pub fn find_case_insensitive(dir: impl AsRef<Path>, target: &str) -> Option<PathBuf> {
    let dir = dir.as_ref();
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Case-insensitive comparison
            if file_name_str.eq_ignore_ascii_case(target) {
                return Some(entry.path());
            }
        }
    }
    
    None
}

/// Watch a directory for changes (for hot reload).
pub struct DirectoryWatcher {
    path: PathBuf,
    // TODO: Implement using notify crate for production
}

impl DirectoryWatcher {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = normalize_path(path);
        if !path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", path.display());
        }
        
        Ok(Self { path })
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_normalize_path() {
        // Test with existing temp directory
        let temp_dir = TempDir::new().unwrap();
        let normalized = normalize_path(temp_dir.path());
        assert!(normalized.is_absolute());
    }
    
    #[test]
    fn test_is_file_locked() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();
        
        // File should not be locked
        assert!(!is_file_locked(&file_path));
        
        // Open file for writing to simulate lock
        let _file = OpenOptions::new()
            .write(true)
            .open(&file_path)
            .unwrap();
        
        // Now it might be locked (depends on Windows file sharing)
        // This test is platform-specific
    }
    
    #[test]
    fn test_case_insensitive_lookup() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("TestFile.txt");
        std::fs::write(&file_path, "test").unwrap();
        
        // Should find file with different case
        let found = find_case_insensitive(temp_dir.path(), "testfile.txt");
        assert!(found.is_some());
        assert_eq!(found.unwrap().file_name().unwrap(), "TestFile.txt");
    }
} 