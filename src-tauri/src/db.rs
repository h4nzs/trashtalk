use rusqlite::Connection;
use anyhow::{Result, Context};
use dirs::home_dir;
use chrono::{DateTime, Utc};

/// Represents a record in the file_logs table.
pub struct FileLog {
    pub id: i32,
    pub file_name: String,
    pub original_path: String,
    pub ghost_path: Option<String>,
    pub download_date: DateTime<Utc>,
    pub status: FileStatus,
}

/// Represents the status of a tracked file.
#[derive(Debug, PartialEq)]
pub enum FileStatus {
    Active,
    Ghost,
}

impl FileStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FileStatus::Active => "ACTIVE",
            FileStatus::Ghost => "GHOST",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "ACTIVE" => Ok(FileStatus::Active),
            "GHOST" => Ok(FileStatus::Ghost),
            _ => anyhow::bail!("Invalid file status: {}", s),
        }
    }
}

/// Initializes the local SQLite database (~/.trashtalk.db) and creates necessary tables.
pub fn init_db() -> Result<Connection> {
    let home = home_dir().context("Could not find home directory")?;
    let db_path = home.join(".trashtalk.db");

    let conn = Connection::open(db_path)
        .context("Failed to open SQLite database")?;

    // Migration: Ensure columns exist (for older installations)
    let _ = conn.execute("ALTER TABLE file_logs ADD COLUMN ghost_path TEXT", []);
    let _ = conn.execute("ALTER TABLE file_logs ADD COLUMN status TEXT DEFAULT 'ACTIVE'", []);

    // Create the file_logs table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_logs (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name       TEXT NOT NULL,
            original_path   TEXT NOT NULL,
            ghost_path      TEXT,
            download_date   TEXT NOT NULL,
            status          TEXT NOT NULL
        )",
        [],
    ).context("Failed to create file_logs table")?;

    // Create the settings table for app configuration
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key             TEXT PRIMARY KEY,
            value           TEXT NOT NULL
        )",
        [],
    ).context("Failed to create settings table")?;

    // Initialize default schedule if not exists
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('schedule_day', 'Friday')",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('schedule_time', '16:00')",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('last_trigger', '1970-01-01T00:00:00Z')",
        [],
    )?;

    Ok(conn)
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<String> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
    let value: String = stmt.query_row([key], |row| row.get(0))?;
    Ok(value)
}

pub fn update_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        (key, value),
    )?;
    Ok(())
}

pub fn log_ghost_file(conn: &Connection, name: &str, original: &str, ghost: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO file_logs (file_name, original_path, ghost_path, download_date, status)
         VALUES (?, ?, ?, ?, ?)",
        (name, original, ghost, Utc::now().to_rfc3339(), "GHOST"),
    ).context("Failed to insert ghost file log")?;
    Ok(())
}

pub fn get_ghost_logs(conn: &Connection) -> Result<Vec<(i32, String, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, file_name, original_path, ghost_path FROM file_logs WHERE status = 'GHOST'")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row?);
    }
    Ok(logs)
}

pub fn delete_log(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM file_logs WHERE id = ?", [id])?;
    Ok(())
}

pub fn clear_ghost_logs(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM file_logs WHERE status = 'GHOST'", [])?;
    Ok(())
}
