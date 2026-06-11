use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "trashtalk")]
#[command(about = "Digital hoarding cleanup tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scans Downloads and lists stale files (> 14 days old) - Dry run
    Scan,
    /// Scans and moves all stale files to the Ghost Folder immediately
    PurgeNow,
    /// Lists all files currently in the Ghost Folder
    GhostList,
    /// Restores a file from the Ghost Folder to its original path by ID
    Restore { id: i32 },
    /// Permanently deletes all files in the Ghost Folder
    GhostEmpty,
    /// Manages the ignore list
    Ignore {
        #[command(subcommand)]
        action: IgnoreAction,
    },
}

#[derive(Subcommand)]
pub enum IgnoreAction {
    /// Adds a path or filename to the ignore list
    Add { path: String },
    /// Lists all ignored paths
    List,
    /// Removes a path from the ignore list
    Remove { path: String },
}

