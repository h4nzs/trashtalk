mod scanner;
mod ghost;
mod cli;
mod db;
mod category;
mod i18n;
mod scheduler;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, IgnoreAction};
use category::FileCategory;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

// Initialize i18n for CLI usage
rust_i18n::i18n!("locales");

#[derive(Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_stale: usize,
    pub total_size_bytes: u64,
    pub breakdown: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize)]
pub struct GhostFile {
    pub id: Option<i32>,
    pub name: String,
    pub original_path: String,
    pub ghost_path: String,
}

#[tauri::command]
async fn run_scan() -> Result<ScanSummary, String> {
    let all_files = scanner::scan_downloads().map_err(|e| e.to_string())?;
    let mut summary = HashMap::new();
    let mut total_stale = 0;
    let mut total_size_bytes = 0;

    for file in all_files {
        if scanner::is_stale(&file, 14).map_err(|e| e.to_string())? {
            total_stale += 1;
            if let Ok(meta) = std::fs::metadata(&file) {
                total_size_bytes += meta.len();
            }
            let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown");
            let extension = file.extension().and_then(|s| s.to_str()).unwrap_or("");
            let cat = category::categorize_file(file_name, extension);
            *summary.entry(cat.as_str().to_string()).or_insert(0) += 1;
        }
    }

    Ok(ScanSummary { total_stale, total_size_bytes, breakdown: summary })
}

#[tauri::command]
async fn run_purge() -> Result<usize, String> {
    let all_files = scanner::scan_downloads().map_err(|e| e.to_string())?;
    let mut moved_count = 0;
    let conn = db::init_db().map_err(|e| e.to_string())?;

    for file in all_files {
        if scanner::is_stale(&file, 14).map_err(|e| e.to_string())? {
            let original_path = file.to_string_lossy().to_string();
            let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
            
            let ghost_path_buf = ghost::move_to_ghost_folder(&file).map_err(|e| e.to_string())?;
            let ghost_path = ghost_path_buf.to_string_lossy().to_string();
            
            db::log_ghost_file(&conn, &file_name, &original_path, &ghost_path).map_err(|e| e.to_string())?;
            moved_count += 1;
        }
    }

    Ok(moved_count)
}

#[tauri::command]
async fn run_custom_purge(time_range_days: i64, categories: Vec<String>) -> Result<usize, String> {
    let all_files = scanner::scan_downloads().map_err(|e| e.to_string())?;
    let mut moved_count = 0;
    let conn = db::init_db().map_err(|e| e.to_string())?;

    for file in all_files {
        if scanner::is_stale(&file, time_range_days).map_err(|e| e.to_string())? {
            let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown");
            let extension = file.extension().and_then(|s| s.to_str()).unwrap_or("");
            let cat = category::categorize_file(file_name, extension);
            
            if categories.contains(&cat.as_str().to_string()) {
                let original_path = file.to_string_lossy().to_string();
                let file_name_string = file_name.to_string();
                
                let ghost_path_buf = ghost::move_to_ghost_folder(&file).map_err(|e| e.to_string())?;
                let ghost_path = ghost_path_buf.to_string_lossy().to_string();
                
                db::log_ghost_file(&conn, &file_name_string, &original_path, &ghost_path).map_err(|e| e.to_string())?;
                moved_count += 1;
            }
        }
    }

    Ok(moved_count)
}

#[tauri::command]
async fn get_ghost_files() -> Result<Vec<GhostFile>, String> {
    let conn = db::init_db().map_err(|e| e.to_string())?;
    let logs = db::get_ghost_logs(&conn).map_err(|e| e.to_string())?;
    
    Ok(logs.into_iter().map(|(id, name, original, ghost)| GhostFile {
        id: Some(id),
        name,
        original_path: original,
        ghost_path: ghost,
    }).collect())
}

#[tauri::command]
async fn restore_ghost_file(id: i32, ghost_path: String, original_path: String) -> Result<(), String> {
    let g_path = PathBuf::from(&ghost_path);
    let o_path = PathBuf::from(&original_path);
    
    ghost::restore_file(&g_path, &o_path).map_err(|e| e.to_string())?;
    
    let conn = db::init_db().map_err(|e| e.to_string())?;
    db::delete_log(&conn, id).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn empty_ghost() -> Result<(), String> {
    ghost::empty_ghost_folder().map_err(|e| e.to_string())?;
    let conn = db::init_db().map_err(|e| e.to_string())?;
    db::clear_ghost_logs(&conn).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_ignore_list() -> Result<Vec<String>, String> {
    i18n::read_ignore_list().map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_ignore_list(list: Vec<String>) -> Result<(), String> {
    i18n::write_ignore_list(list).map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize)]
pub struct AppSchedule {
    pub day: String,
    pub time: String,
}

#[tauri::command]
async fn get_schedule() -> Result<AppSchedule, String> {
    let conn = db::init_db().map_err(|e| e.to_string())?;
    let day = db::get_setting(&conn, "schedule_day").unwrap_or_else(|_| "Friday".to_string());
    let time = db::get_setting(&conn, "schedule_time").unwrap_or_else(|_| "16:00".to_string());
    Ok(AppSchedule { day, time })
}

#[tauri::command]
async fn save_schedule(schedule: AppSchedule) -> Result<(), String> {
    let conn = db::init_db().map_err(|e| e.to_string())?;
    db::update_setting(&conn, "schedule_day", &schedule.day).map_err(|e| e.to_string())?;
    db::update_setting(&conn, "schedule_time", &schedule.time).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() -> Result<()> {
    // Silence libayatana deprecation warnings on Linux
    unsafe {
        std::env::set_var("G_MESSAGES_DEBUG", "none");
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let cli = Cli::parse();
        return run_cli(cli);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .setup(|app| {
            db::init_db()?;

            // 1. Setup System Tray
            let show_manager = MenuItemBuilder::with_id("show_manager", "Show Manager").build(app)?;
            let scan_now = MenuItemBuilder::with_id("scan_now", "Scan Now").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            
            let menu = MenuBuilder::new(app)
                .items(&[&show_manager, &scan_now, &quit])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show_manager" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.unminimize().unwrap();
                        window.set_focus().unwrap();
                    }
                    "scan_now" => {
                        let window = app.get_webview_window("main").unwrap();
                        window.show().unwrap();
                        window.unminimize().unwrap();
                        window.set_focus().unwrap();
                        // We could trigger a JS event here if needed
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // 2. Spawn background scheduler
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::spawn_scheduler(handle).await;
            });
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            run_scan,
            run_purge,
            run_custom_purge,
            get_ghost_files,
            restore_ghost_file,
            empty_ghost,
            get_ignore_list,
            save_ignore_list,
            get_schedule,
            save_schedule
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

fn run_cli(cli: Cli) -> Result<()> {
    use cli_table::{print_stdout, Table, WithTitle};
    #[derive(Table)]
    struct SummaryRow {
        #[table(title = "Category")]
        category: String,
        #[table(title = "Count")]
        count: usize,
    }

    let conn = db::init_db()?;

    match cli.command {
        Commands::Scan => {
            println!("{}", rust_i18n::t!("scan_header"));
            let all_files = scanner::scan_downloads()?;
            let mut summary = HashMap::new();
            let mut total_stale = 0;

            for file in all_files {
                if scanner::is_stale(&file, 14)? {
                    total_stale += 1;
                    let file_name = file.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown");
                    let extension = file.extension().and_then(|s| s.to_str()).unwrap_or("");
                    let cat = category::categorize_file(file_name, extension);
                    *summary.entry(cat.as_str().to_string()).or_insert(0) += 1;
                }
            }

            let mut table_data = Vec::new();
            let categories = vec![
                FileCategory::StaleInstallers,
                FileCategory::SocialMedia,
                FileCategory::HeavyVideos,
                FileCategory::WorkDocuments,
                FileCategory::Archives,
                FileCategory::DesignFiles,
                FileCategory::MemesAndGifs,
                FileCategory::Unknown,
            ];

            for cat in categories {
                if let Some(&count) = summary.get(cat.as_str()) {
                    table_data.push(SummaryRow { category: cat.as_str().to_string(), count });
                }
            }

            if total_stale > 0 { print_stdout(table_data.with_title())?; }
            println!("--------------------------------------------------");
            println!("{}", rust_i18n::t!("scan_complete", count = total_stale));
        }
        Commands::PurgeNow => {
            let all_files = scanner::scan_downloads()?;
            let mut moved_count = 0;
            for file in all_files {
                if scanner::is_stale(&file, 14)? {
                    let original = file.to_string_lossy().to_string();
                    let name = file.file_name().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
                    let ghost = ghost::move_to_ghost_folder(&file)?.to_string_lossy().to_string();
                    db::log_ghost_file(&conn, &name, &original, &ghost)?;
                    moved_count += 1;
                }
            }
            println!("✅ Purge complete. Moved {} files to Ghost Folder.", moved_count);
        }
        Commands::GhostList => {
            let logs = db::get_ghost_logs(&conn)?;
            if logs.is_empty() {
                println!("Ghost Folder is empty.");
            } else {
                for (id, name, original, _) in logs {
                    println!("[{}] {} (Original: {})", id, name, original);
                }
            }
        }
        Commands::Restore { id } => {
            let logs = db::get_ghost_logs(&conn)?;
            if let Some((_, _, original, ghost_path)) = logs.into_iter().find(|(log_id, _, _, _)| *log_id == id) {
                let g_path = PathBuf::from(&ghost_path);
                let o_path = PathBuf::from(&original);
                ghost::restore_file(&g_path, &o_path)?;
                db::delete_log(&conn, id)?;
                println!("✅ Restored file to {}", original);
            } else {
                println!("❌ File with ID {} not found in Ghost logs.", id);
            }
        }
        Commands::GhostEmpty => {
            ghost::empty_ghost_folder()?;
            db::clear_ghost_logs(&conn)?;
            println!("✅ Ghost folder emptied successfully.");
        }
        Commands::Ignore { action } => {
            let mut list = i18n::read_ignore_list()?;
            match action {
                IgnoreAction::Add { path } => {
                    if !list.contains(&path) {
                        list.push(path.clone());
                        i18n::write_ignore_list(list)?;
                        println!("✅ Added '{}' to ignore list.", path);
                    } else {
                        println!("⚠️ '{}' is already in the ignore list.", path);
                    }
                }
                IgnoreAction::List => {
                    if list.is_empty() {
                        println!("Ignore list is empty.");
                    } else {
                        println!("Ignored paths:");
                        for item in list {
                            println!("- {}", item);
                        }
                    }
                }
                IgnoreAction::Remove { path } => {
                    let initial_len = list.len();
                    list.retain(|x| x != &path);
                    if list.len() < initial_len {
                        i18n::write_ignore_list(list)?;
                        println!("✅ Removed '{}' from ignore list.", path);
                    } else {
                        println!("⚠️ '{}' not found in ignore list.", path);
                    }
                }
            }
        }
    }
    Ok(())
}
