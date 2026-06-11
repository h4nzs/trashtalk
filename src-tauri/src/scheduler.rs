use crate::db;
use anyhow::{Result, Context};
use chrono::{DateTime, Datelike, Local, Timelike, Utc, Weekday};
use std::str::FromStr;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

/// Starts the background scheduler loop.
pub async fn spawn_scheduler(handle: AppHandle) {
    println!("⏰ Scheduler thread started.");
    
    // Initial check for missed schedule on boot
    if let Err(e) = check_and_trigger_if_missed(&handle).await {
        eprintln!("Error checking missed schedule: {}", e);
    }

    loop {
        // Sleep for 1 minute between checks to save resources
        sleep(Duration::from_secs(60)).await;

        if let Err(e) = check_current_schedule(&handle).await {
            eprintln!("Scheduler error: {}", e);
        }
    }
}

async fn check_current_schedule(handle: &AppHandle) -> Result<()> {
    let conn = db::init_db()?;
    let schedule_day = db::get_setting(&conn, "schedule_day").unwrap_or_else(|_| "Friday".to_string());
    let schedule_time = db::get_setting(&conn, "schedule_time").unwrap_or_else(|_| "16:00".to_string());

    let now = Local::now();
    let current_day = now.weekday().to_string();
    let current_time = format!("{:02}:{:02}", now.hour(), now.minute());

    if current_day.to_lowercase() == schedule_day.to_lowercase() && current_time == schedule_time {
        // Check if we already triggered in this minute to avoid double prompts
        let last_trigger_str = db::get_setting(&conn, "last_trigger").unwrap_or_default();
        if let Ok(last_trigger) = DateTime::parse_from_rfc3339(&last_trigger_str) {
            let last_trigger_local = last_trigger.with_timezone(&Local);
            if last_trigger_local.date_naive() == now.date_naive() && 
               last_trigger_local.hour() == now.hour() && 
               last_trigger_local.minute() == now.minute() {
                return Ok(());
            }
        }

        println!("🎯 Scheduled time reached ({} at {}). Triggering prompt.", schedule_day, schedule_time);
        trigger_prompt(handle, &conn)?;
    }

    Ok(())
}

async fn check_and_trigger_if_missed(handle: &AppHandle) -> Result<()> {
    let conn = db::init_db()?;
    let schedule_day_str = db::get_setting(&conn, "schedule_day").unwrap_or_else(|_| "Friday".to_string());
    let schedule_time_str = db::get_setting(&conn, "schedule_time").unwrap_or_else(|_| "16:00".to_string());
    let last_trigger_str = db::get_setting(&conn, "last_trigger").unwrap_or_default();

    let target_weekday = Weekday::from_str(&schedule_day_str).map_err(|_| anyhow::anyhow!("Invalid weekday in settings"))?;
    let last_trigger = DateTime::parse_from_rfc3339(&last_trigger_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| DateTime::from_timestamp(0, 0).unwrap().with_timezone(&Utc));

    let now = Utc::now();
    
    // Logic: If last trigger was more than 7 days ago, or if we passed the target day/time since last trigger
    // Simplification: Check if the most recent target timestamp in the past is AFTER the last_trigger timestamp.
    
    let last_target = get_last_occurrence(target_weekday, &schedule_time_str)?;
    
    if last_target > last_trigger && last_target < now {
        println!("⚠️ Missed schedule detected! Last scheduled was {:?}, but last trigger was {:?}.", last_target, last_trigger);
        trigger_prompt(handle, &conn)?;
    }

    Ok(())
}

fn trigger_prompt(handle: &AppHandle, conn: &rusqlite::Connection) -> Result<()> {
    if let Some(window) = handle.get_webview_window("main") {
        window.show().context("Failed to show window")?;
        window.unminimize().context("Failed to unminimize window")?;
        window.set_focus().context("Failed to focus window")?;
        
        // Update last trigger time
        db::update_setting(conn, "last_trigger", &Utc::now().to_rfc3339())?;
    }
    Ok(())
}

/// Helper to find the most recent occurrence of a specific weekday and time in the past (UTC).
fn get_last_occurrence(target_weekday: Weekday, time_str: &str) -> Result<DateTime<Utc>> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 { anyhow::bail!("Invalid time format"); }
    let hour: u32 = parts[0].parse()?;
    let min: u32 = parts[1].parse()?;

    let mut check_date = Local::now();
    
    // Go back day by day until we hit the target weekday
    while check_date.weekday() != target_weekday {
        check_date = check_date - chrono::Duration::days(1);
    }
    
    // Set the specific time
    let last_occurrence_local = check_date
        .with_hour(hour).context("Invalid hour")?
        .with_minute(min).context("Invalid minute")?
        .with_second(0).context("Invalid second")?
        .with_nanosecond(0).context("Invalid nano")?;

    // If that occurrence today is actually in the future, go back 7 days
    let final_occurrence = if last_occurrence_local > Local::now() {
        last_occurrence_local - chrono::Duration::days(7)
    } else {
        last_occurrence_local
    };

    Ok(final_occurrence.with_timezone(&Utc))
}
