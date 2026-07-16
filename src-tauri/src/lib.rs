mod autostart_manager;
mod countdown_store;
mod window_utils;

use countdown_store::{Countdown, CountdownStore};
use std::sync::Mutex;
use tauri::{Manager, State};

pub struct AppState {
    pub store: CountdownStore,
}

#[tauri::command]
fn get_all_countdowns(
    state: State<'_, Mutex<AppState>>,
) -> Result<countdown_store::AppData, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    Ok(app.store.get_all())
}

#[tauri::command]
fn save_countdown(
    state: State<'_, Mutex<AppState>>,
    countdown: Countdown,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    app.store
        .save_countdown(countdown)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_countdown(state: State<'_, Mutex<AppState>>, id: String) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    app.store.delete_countdown(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_window_position(
    state: State<'_, Mutex<AppState>>,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    app.store
        .save_window_position(x, y)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    autostart_manager::set_autostart(enabled)
}

#[tauri::command]
fn get_autostart() -> Result<bool, String> {
    autostart_manager::get_autostart()
}

pub fn run() {
    let store = CountdownStore::new().expect("Failed to initialize data store");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState { store }))
        .setup(|app| {
            let window = app
                .get_webview_window("main")
                .expect("main window not found");
            let state = app.state::<Mutex<AppState>>();
            let app_state = state.lock().unwrap();
            let pos = &app_state.store.get_all().settings.window_position;
            window_utils::validate_and_position(&window, pos.x, pos.y);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_countdowns,
            save_countdown,
            delete_countdown,
            save_window_position,
            set_autostart,
            get_autostart,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DeskCountdown");
}
