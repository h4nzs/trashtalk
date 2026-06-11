use std::path::{Path, PathBuf};
use walkdir::{WalkDir, DirEntry};
use anyhow::{Result, Context};
use dirs::download_dir;
use filetime::FileTime;
use chrono::{DateTime, Utc, Duration};
use crate::i18n;

/// Scans the ~/Downloads directory recursively for files.
/// Gracefully skips files with permission errors or other access issues.
/// Automatically skips paths listed in ~/.trashtalkignore.
pub fn scan_downloads() -> Result<Vec<PathBuf>> {
    let downloads_path = download_dir()
        .context("Could not find the Downloads directory")?;

    let ignore_list = i18n::get_ignore_paths();
    let mut files = Vec::new();

    let walker = WalkDir::new(&downloads_path)
        .into_iter()
        .filter_entry(|e| !is_ignored(e, &ignore_list));

    for entry in walker {
        match entry {
            Ok(e) => {
                if e.file_type().is_file() {
                    files.push(e.path().to_path_buf());
                }
            }
            Err(err) => {
                // Graceful error handling for Permission Denied and other transient errors
                if let Some(io_err) = err.io_error() {
                    match io_err.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            eprintln!("Warning: Permission denied for path: {:?}. Skipping...", err.path());
                            continue;
                        }
                        _ => {
                            eprintln!("Warning: Error accessing path: {:?}. Error: {}. Skipping...", err.path(), err);
                            continue;
                        }
                    }
                }
                eprintln!("Warning: Unexpected error during scan: {}. Skipping...", err);
            }
        }
    }

    Ok(files)
}

/// Checks if a directory entry should be ignored.
fn is_ignored(entry: &DirEntry, ignore_list: &[PathBuf]) -> bool {
    let path = entry.path();
    
    for ignore_path in ignore_list {
        // 1. Check for exact match (absolute or relative to current path)
        if path == ignore_path {
            return true;
        }

        // 2. Check if the path is inside an ignored directory
        if path.starts_with(ignore_path) {
            return true;
        }

        // 3. Check for filename match (if ignore_path is just a name)
        if let Some(file_name) = path.file_name() {
            if file_name == ignore_path {
                return true;
            }
        }
    }

    false
}

/// Checks if a file is 'stale' (last modified more than `days` ago).
/// Returns true if stale, false otherwise.
pub fn is_stale(path: &Path, days: i64) -> Result<bool> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for {:?}", path))?;
    
    let mtime = FileTime::from_last_modification_time(&metadata);
    let mtime_seconds = mtime.unix_seconds();
    
    let mtime_datetime = DateTime::from_timestamp(mtime_seconds, 0)
        .context("Invalid modification time timestamp")?;
    
    let now = Utc::now();
    let threshold = now - Duration::days(days);
    
    Ok(mtime_datetime < threshold)
}
