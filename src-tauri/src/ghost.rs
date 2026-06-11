use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context, anyhow};
use dirs::home_dir;
use chrono::Local;

/// Moves a file to the 'Ghost Folder' (~/.trashtalk_ghost).
/// If a collision occurs, appends a timestamp to the filename.
/// Returns the new path of the file in the ghost folder.
pub fn move_to_ghost_folder(file_path: &Path) -> Result<PathBuf> {
    // 1. Get/Create Ghost Folder Path
    let home = home_dir().context("Could not find home directory")?;
    let ghost_dir = home.join(".trashtalk_ghost");

    if !ghost_dir.exists() {
        fs::create_dir_all(&ghost_dir)
            .context("Failed to create ghost folder")?;
    }

    // 2. Prepare Destination Path
    let file_name = file_path.file_name()
        .ok_or_else(|| anyhow!("Invalid file path: {:?}", file_path))?;
    
    let mut dest_path = ghost_dir.join(file_name);

    // 3. Handle Name Collisions
    if dest_path.exists() {
        let stem = Path::new(file_name).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = Path::new(file_name).extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let new_name = format!("{}_{}{}", stem, timestamp, extension);
        dest_path = ghost_dir.join(new_name);
    }

    // 4. Move the File
    fs::rename(file_path, &dest_path)
        .with_context(|| format!("Failed to move file from {:?} to {:?}", file_path, &dest_path))?;

    println!("Successfully moved {:?} to ghost folder as {:?}", file_name, dest_path.file_name().unwrap());

    Ok(dest_path)
}

/// Lists all files currently in the ghost folder.
pub fn list_ghost_files() -> Result<Vec<PathBuf>> {
    let home = home_dir().context("Could not find home directory")?;
    let ghost_dir = home.join(".trashtalk_ghost");

    if !ghost_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(ghost_dir).context("Failed to read ghost folder")? {
        let entry = entry.context("Failed to read directory entry")?;
        if entry.path().is_file() {
            files.push(entry.path());
        }
    }

    Ok(files)
}

/// Restores a file from ghost folder to its original path.
pub fn restore_file(ghost_path: &Path, original_path: &Path) -> Result<()> {
    if !ghost_path.exists() {
        return Err(anyhow!("Ghost file does not exist: {:?}", ghost_path));
    }

    // Ensure original directory exists
    if let Some(parent) = original_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).context("Failed to recreate original directory")?;
        }
    }

    // Handle collision at original path
    let mut final_dest = original_path.to_path_buf();
    if final_dest.exists() {
        let stem = original_path.file_stem().and_then(|s| s.to_str()).unwrap_or("restored");
        let ext = original_path.extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        final_dest.set_file_name(format!("{}_restored_{}{}", stem, timestamp, ext));
    }

    fs::rename(ghost_path, &final_dest)
        .with_context(|| format!("Failed to restore file to {:?}", final_dest))?;

    Ok(())
}

/// Permanently deletes all files in the ghost folder.
pub fn empty_ghost_folder() -> Result<()> {
    let home = home_dir().context("Could not find home directory")?;
    let ghost_dir = home.join(".trashtalk_ghost");

    if ghost_dir.exists() {
        fs::remove_dir_all(&ghost_dir).context("Failed to delete ghost folder")?;
        fs::create_dir_all(&ghost_dir).context("Failed to recreate empty ghost folder")?;
    }

    Ok(())
}
