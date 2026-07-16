use std::sync::Mutex;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, CheckMenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn create_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_show = MenuItemBuilder::with_id("toggle_show", "显示/隐藏").build(app)?;
    let particles = CheckMenuItemBuilder::with_id("particles", "特效").build(app)?;
    let autostart = CheckMenuItemBuilder::with_id("autostart", "开机自启").build(app)?;
    let about = MenuItemBuilder::with_id("about", "关于").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle_show)
        .item(&particles)
        .item(&autostart)
        .item(&about)
        .item(&quit)
        .build()?;

    // Initialize checkmark states
    let state = app.state::<Mutex<crate::AppState>>();
    if let Ok(app_state) = state.lock() {
        let settings = app_state.store.get_all().settings;
        let _ = particles.set_checked(settings.particles_enabled);
        let _ = autostart.set_checked(settings.autostart);
    }

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("桌面倒计时 - 左键切换显示")
        .menu(&menu)
        .on_menu_event(move |app, event| {
            match event.id().as_ref() {
                "toggle_show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                        }
                    }
                }
                "particles" => {
                    let state = app.state::<Mutex<crate::AppState>>();
                    let lock_result = state.lock();
                    if let Ok(app_state) = lock_result {
                        let current = app_state.store.get_all().settings.particles_enabled;
                        let new_val = !current;
                        let _ = app_state.store.update_settings(|s| {
                            s.particles_enabled = new_val;
                        });
                        let _ = app.emit("particles-toggled", new_val);
                    }
                }
                "autostart" => {
                    let state = app.state::<Mutex<crate::AppState>>();
                    let lock_result = state.lock();
                    if let Ok(app_state) = lock_result {
                        let current = app_state.store.get_all().settings.autostart;
                        let new_val = !current;
                        if crate::autostart_manager::set_autostart(new_val).is_ok() {
                            let _ = app_state.store.update_settings(|s| {
                                s.autostart = new_val;
                            });
                        }
                    }
                }
                "about" => {
                    use tauri_plugin_dialog::DialogExt;
                    app.dialog()
                        .message("DeskCountdown 桌面倒计时 v0.1.0")
                        .title("关于")
                        .show(|_| {});
                }
                "quit" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let pos = window.outer_position().unwrap_or_default();
                        let state = app.state::<Mutex<crate::AppState>>();
                        let lock_result = state.lock();
                        if let Ok(app_state) = lock_result {
                            let _ = app_state.store.save_window_position(pos.x, pos.y);
                        }
                    }
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
