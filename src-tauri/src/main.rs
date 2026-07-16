#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

extern "system" {
    fn CreateMutexW(
        lp_mutex_attributes: *const std::ffi::c_void,
        b_initial_owner: i32,
        lp_name: *const u16,
    ) -> *mut std::ffi::c_void;

    fn GetLastError() -> u32;

    fn FindWindowW(
        lp_class_name: *const u16,
        lp_window_name: *const u16,
    ) -> *mut std::ffi::c_void;

    fn ShowWindow(hwnd: *mut std::ffi::c_void, n_cmd_show: i32);

    fn SetForegroundWindow(hwnd: *mut std::ffi::c_void) -> i32;
}

const ERROR_ALREADY_EXISTS: u32 = 183;
const SW_RESTORE: i32 = 9;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn main() {
    let name = to_wide("DeskCountdown/SingleInstance");

    unsafe {
        let _handle = CreateMutexW(std::ptr::null(), 1, name.as_ptr());

        if GetLastError() == ERROR_ALREADY_EXISTS {
            let title = to_wide("桌面倒计时");
            let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
            if !hwnd.is_null() {
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            }
            return;
        }

        // Mutex handle auto-closed when process exits
    }

    deskcountdown::run()
}
