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
}
