//! Utility modules for Pika-Plot.

#[cfg(target_os = "windows")]
pub mod windows_fs;

pub mod paths {
    use std::path::{Path, PathBuf};
    
    /// Normalize a path for the current platform.
    pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            crate::utils::windows_fs::normalize_path(path)
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            path.as_ref().to_path_buf()
        }
    }
} 