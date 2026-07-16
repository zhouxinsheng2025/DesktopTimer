use winreg::enums::*;
use winreg::RegKey;

const REG_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const REG_NAME: &str = "DeskCountdown";

pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_PATH, KEY_WRITE)
        .map_err(|e| format!("Failed to open registry: {}", e))?;

    if enabled {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        let exe_str = exe_path.to_string_lossy().to_string();
        run_key
            .set_value(REG_NAME, &exe_str)
            .map_err(|e| format!("Failed to set registry value: {}", e))?;
    } else {
        run_key
            .delete_value(REG_NAME)
            .map_err(|e| format!("Failed to delete registry value: {}", e))?;
    }

    Ok(())
}

pub fn get_autostart() -> Result<bool, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(REG_PATH, KEY_READ)
        .map_err(|e| format!("Failed to open registry: {}", e))?;

    match run_key.get_value::<String, _>(REG_NAME) {
        Ok(_) => Ok(true),
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Failed to read registry: {}", e)),
    }
}
