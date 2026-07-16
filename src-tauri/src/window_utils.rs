use tauri::{PhysicalPosition, WebviewWindow};

pub fn validate_and_position(window: &WebviewWindow, saved_x: i32, saved_y: i32) {
    let monitors = window.available_monitors().unwrap_or_default();
    let is_visible = monitors.iter().any(|m| {
        let size = m.size();
        let pos = m.position();
        saved_x >= pos.x
            && saved_y >= pos.y
            && saved_x < pos.x + size.width as i32
            && saved_y < pos.y + size.height as i32
    });

    if is_visible && (saved_x != 0 || saved_y != 0) {
        let _ = window.set_position(PhysicalPosition::new(saved_x, saved_y));
    } else {
        // Default: bottom-right corner of primary monitor's working area
        if let Some(monitor) = window.primary_monitor().unwrap_or_default() {
            let size = monitor.size();
            let pos = monitor.position();
            let win_size = window.outer_size().unwrap();
            let x = pos.x + size.width as i32 - win_size.width as i32 - 40;
            let y = pos.y + size.height as i32 - win_size.height as i32 - 40;
            let _ = window.set_position(PhysicalPosition::new(x.max(pos.x), y.max(pos.y)));
        }
    }
}
