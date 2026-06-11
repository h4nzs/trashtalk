use std::path::PathBuf;
use std::fs;
use anyhow::{Result, Context};
use dirs::home_dir;

/// Reads the ignore list from ~/.trashtalkignore.
pub fn read_ignore_list() -> Result<Vec<String>> {
    let home = home_dir().context("Could not find home directory")?;
    let ignore_file = home.join(".trashtalkignore");

    if !ignore_file.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(ignore_file)
        .context("Failed to read .trashtalkignore")?;

    let mut list = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            list.push(trimmed.to_string());
        }
    }

    Ok(list)
}

/// Overwrites the ignore list in ~/.trashtalkignore.
pub fn write_ignore_list(list: Vec<String>) -> Result<()> {
    let home = home_dir().context("Could not find home directory")?;
    let ignore_file = home.join(".trashtalkignore");

    let content = list.join("\n");
    fs::write(ignore_file, content)
        .context("Failed to write .trashtalkignore")?;

    Ok(())
}

/// Internal helper for scanner to get PathBufs
pub fn get_ignore_paths() -> Vec<PathBuf> {
    read_ignore_list().unwrap_or_default()
        .into_iter()
        .map(PathBuf::from)
        .collect()
}
