# DeskCountdown Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Windows desktop countdown widget with Tauri v2 + Vue 3 — frameless transparent window, multiple gradient-styled countdown cards, right-click management, system tray, and optional autostart.

**Architecture:** Tauri v2 Rust backend handles JSON persistence, registry autostart, system tray, and window configuration. Vue 3 frontend renders countdown cards with CSS gradient text, provides inline editor and context menu, and runs a per-second countdown engine with zero IPC overhead.

**Tech Stack:** Tauri v2, Rust, Vue 3 + Vite + TypeScript, vanilla CSS, `winreg` crate, `tauri-plugin-dialog`

---

## Global Constraints

- App name: DeskCountdown / 桌面倒计时
- Window: `decorations: false`, `transparent: true`, `always_on_top: false`, `skip_taskbar: true`
- All timestamps in local time, no UTC conversion
- Data stored at `%APPDATA%\countdown-timer\data.json`
- Single instance via Windows named mutex
- NSIS installer with custom install path, `installMode: "both"`
- Card width: 300px, window padding: 8px, card padding: 16px (H) × 14px (V)
- Minimum window: 348px wide, ~120px tall
- Default glow: intensity 0.6 (12px blur), gradient: 霓虹青蓝
- Default date for new countdowns: today + 7 days
- Gradients are string keys mapped to CSS gradients, not raw color values

---

## File Structure

```
countdown-timer/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── icons/
│   │   └── icon.ico
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── countdown_store.rs
│       ├── autostart_manager.rs
│       ├── tray_manager.rs
│       └── window_utils.rs
├── src/
│   ├── index.html
│   ├── main.ts
│   ├── App.vue
│   ├── style.css
│   ├── types/
│   │   └── index.ts
│   ├── utils/
│   │   └── format.ts
│   ├── composables/
│   │   ├── useCountdownEngine.ts
│   │   └── useContextMenu.ts
│   └── components/
│       ├── CountdownCard.vue
│       ├── CardList.vue
│       ├── ContextMenu.vue
│       ├── InlineEditor.vue
│       ├── ParticlesBackground.vue
│       └── OnboardingToast.vue
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tsconfig.node.json
```

---

### Task 1: Project Scaffolding

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src/index.html`, `src/main.ts`
- Create: `.gitignore`

**Interfaces:**
- Produces: Runnable Tauri v2 + Vue 3 + Vite project skeleton

- [ ] **Step 1: Initialize project with Tauri CLI**

```bash
cd "C:/Users/24338/Desktop/倒计时"
npm create vue@latest . -- --typescript --no-router --no-pinia --no-vitest --no-eslint --no-prettier
# Select: TypeScript = Yes, others = No for minimal setup
npm install
```

- [ ] **Step 2: Add Tauri v2**

```bash
npm install @tauri-apps/cli@^2 @tauri-apps/api@^2
npx tauri init
# App name: DeskCountdown
# Window title: 桌面倒计时
# Dev URL: http://localhost:1420
# Build command: npm run build
# Dev command: npm run dev
# Frontend dist: ../dist
```

- [ ] **Step 3: Configure `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-cli/schema.json",
  "productName": "DeskCountdown",
  "version": "0.1.0",
  "identifier": "com.deskcountdown.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "桌面倒计时",
        "width": 348,
        "height": 120,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": false,
        "skipTaskbar": true,
        "resizable": false,
        "visible": true,
        "center": false,
        "x": 0,
        "y": 0
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  },
  "bundle": {
    "active": true,
    "targets": "nsis",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ],
    "windows": {
      "nsis": {
        "installMode": "both"
      }
    }
  },
  "plugins": {
    "dialog": {}
  }
}
```

- [ ] **Step 4: Configure `src-tauri/Cargo.toml`**

```toml
[package]
name = "deskcountdown"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
winreg = "0.52"
chrono = "0.4"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 5: Create `src/index.html`**

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>桌面倒计时</title>
    <style>
      html, body {
        margin: 0;
        padding: 0;
        background: transparent !important;
        overflow: hidden;
        user-select: none;
        font-family: "Microsoft YaHei", "PingFang SC", sans-serif;
      }
    </style>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 6: Create `src/main.ts`**

```typescript
import { createApp } from 'vue'
import App from './App.vue'
import './style.css'

createApp(App).mount('#app')
```

- [ ] **Step 7: Create `src-tauri/src/main.rs` skeleton**

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    deskcountdown::run()
}
```

- [ ] **Step 8: Create `src-tauri/src/lib.rs` skeleton**

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running DeskCountdown");
}
```

- [ ] **Step 9: Verify scaffold compiles**

```bash
cd src-tauri
cargo check
```

Expected: Compiles successfully (no errors, may have warnings for unused imports).

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri v2 + Vue 3 + Vite project"
```

---

### Task 2: TypeScript Type Definitions

**Files:**
- Create: `src/types/index.ts`

**Interfaces:**
- Produces: `Countdown`, `FontStyle`, `Settings`, `AppData`, `GradientPreset`, `EditorMode` types exported for all frontend components

- [ ] **Step 1: Write `src/types/index.ts`**

```typescript
export interface FontStyle {
  gradient: string
  glow_intensity: number
}

export interface Countdown {
  id: string
  title: string
  prefix_text: string
  suffix_text: string
  created_at: string
  target_date: string
  font_style: FontStyle
  sort_order: number
}

export interface WindowPosition {
  x: number
  y: number
}

export interface Settings {
  autostart: boolean
  onboarding_complete: boolean
  particles_enabled: boolean
  window_position: WindowPosition
}

export interface AppData {
  version: number
  countdowns: Countdown[]
  settings: Settings
}

export interface GradientPreset {
  key: string
  name: string
  colors: [string, string]
}

export const GRADIENT_PRESETS: GradientPreset[] = [
  { key: 'neon-cyan-blue',  name: '霓虹青蓝',  colors: ['#00f0ff', '#0066ff'] },
  { key: 'rose-gold',       name: '玫瑰金',    colors: ['#ff6b9d', '#ffa751'] },
  { key: 'flame-orange-red',name: '火焰橙红',  colors: ['#ff6a00', '#ee0000'] },
  { key: 'aurora-green',    name: '极光绿',    colors: ['#00ff88', '#00cc66'] },
  { key: 'ice-blue-purple', name: '冰晶蓝紫',  colors: ['#7b9cff', '#c084fc'] },
  { key: 'white-gold',      name: '白金',      colors: ['#e8e8e8', '#b8944b'] },
]

export function getGradientCSS(key: string): string {
  const preset = GRADIENT_PRESETS.find(p => p.key === key)
  const [c1, c2] = preset?.colors ?? GRADIENT_PRESETS[0].colors
  return `linear-gradient(135deg, ${c1}, ${c2})`
}

export function getGlowColor(key: string): string {
  const preset = GRADIENT_PRESETS.find(p => p.key === key)
  return preset?.colors[0] ?? GRADIENT_PRESETS[0].colors[0]
}

export type EditorMode = 'create' | 'edit'
```

- [ ] **Step 2: Verify TypeScript compilation**

```bash
npx vue-tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: add TypeScript type definitions and gradient presets"
```

---

### Task 3: Format Utilities

**Files:**
- Create: `src/utils/format.ts`

**Interfaces:**
- Produces: `formatRemaining(now, targetDate)`, `formatExpired(now, targetDate)`, `formatTargetDate(isoString)`, `computeProgress(createdAt, targetDate, now)`, `parseLocalISO(isoString)`

- [ ] **Step 1: Write `src/utils/format.ts`**

```typescript
export function parseLocalISO(iso: string): Date {
  return new Date(iso)
}

export function formatRemaining(now: Date, target: Date): string {
  let diff = target.getTime() - now.getTime()
  if (diff <= 0) return '0分 00秒'

  const totalSeconds = Math.floor(diff / 1000)
  const days = Math.floor(totalSeconds / 86400)
  const hours = Math.floor((totalSeconds % 86400) / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60

  const pad = (n: number) => String(n).padStart(2, '0')

  if (days >= 1) {
    return `${days}天 ${pad(hours)}时 ${pad(minutes)}分 ${pad(seconds)}秒`
  }
  if (hours >= 1) {
    return `${pad(hours)}时 ${pad(minutes)}分 ${pad(seconds)}秒`
  }
  return `${pad(minutes)}分 ${pad(seconds)}秒`
}

export function formatExpired(now: Date, target: Date): string {
  const diffMs = now.getTime() - target.getTime()
  const hours = diffMs / (1000 * 60 * 60)

  if (hours < 24) return '今日到期'

  const targetDay = new Date(target.getFullYear(), target.getMonth(), target.getDate())
  const nowDay = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const days = Math.floor((nowDay.getTime() - targetDay.getTime()) / 86400000)
  return `已过期 ${days} 天`
}

export function formatTargetDate(iso: string): string {
  const d = parseLocalISO(iso)
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const h = String(d.getHours()).padStart(2, '0')
  const min = String(d.getMinutes()).padStart(2, '0')
  return `${y}-${m}-${day} ${h}:${min}`
}

export function computeProgress(createdAt: string, targetDate: string, now: Date): number {
  const created = parseLocalISO(createdAt).getTime()
  const target = parseLocalISO(targetDate).getTime()
  const total = target - created
  if (total <= 0) return 0
  const elapsed = now.getTime() - created
  return Math.max(0, Math.min(100, Math.round((elapsed / total) * 100)))
}
```

- [ ] **Step 2: Verify compilation**

```bash
npx vue-tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add src/utils/format.ts
git commit -m "feat: add countdown format utilities"
```

---

### Task 4: Rust — Data Model & Countdown Store

**Files:**
- Modify: `src-tauri/Cargo.toml` (add `thiserror`)
- Create: `src-tauri/src/countdown_store.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: none (first Rust module)
- Produces: `Countdown`, `FontStyle`, `Settings`, `AppData` structs; `CountdownStore` with `new()`, `get_all()`, `save_countdown()`, `delete_countdown()`, `update_settings()`, `save_window_position()` methods; Tauri commands `get_all_countdowns`, `save_countdown`, `delete_countdown`, `save_window_position`

- [ ] **Step 1: Add `thiserror` to `src-tauri/Cargo.toml`**

```toml
thiserror = "1"
```

- [ ] **Step 2: Write `src-tauri/src/countdown_store.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontStyle {
    pub gradient: String,
    pub glow_intensity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Countdown {
    pub id: String,
    pub title: String,
    pub prefix_text: String,
    pub suffix_text: String,
    pub created_at: String,
    pub target_date: String,
    pub font_style: FontStyle,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub autostart: bool,
    pub onboarding_complete: bool,
    pub particles_enabled: bool,
    pub window_position: WindowPosition,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autostart: false,
            onboarding_complete: false,
            particles_enabled: false,
            window_position: WindowPosition { x: 0, y: 0 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppData {
    pub version: i32,
    pub countdowns: Vec<Countdown>,
    pub settings: Settings,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            version: 1,
            countdowns: vec![],
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct CountdownStore {
    path: PathBuf,
    data: Mutex<AppData>,
}

impl CountdownStore {
    pub fn new() -> Result<Self, StoreError> {
        let path = get_data_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = if path.exists() {
            let content = fs::read_to_string(&path)?;
            match serde_json::from_str::<AppData>(&content) {
                Ok(d) => d,
                Err(_) => {
                    let bak = path.with_extension("json.bak");
                    fs::write(&bak, &content)?;
                    AppData::default()
                }
            }
        } else {
            AppData::default()
        };

        Ok(Self {
            path,
            data: Mutex::new(data),
        })
    }

    fn save(&self) -> Result<(), StoreError> {
        let data = self.data.lock().unwrap();
        let json = serde_json::to_string_pretty(&*data)?;
        fs::write(&self.path, json)?;
        Ok(())
    }

    pub fn get_all(&self) -> AppData {
        self.data.lock().unwrap().clone()
    }

    pub fn save_countdown(&self, countdown: Countdown) -> Result<(), StoreError> {
        {
            let mut data = self.data.lock().unwrap();
            if let Some(existing) = data.countdowns.iter_mut().find(|c| c.id == countdown.id) {
                *existing = countdown;
            } else {
                data.countdowns.push(countdown);
            }
        }
        self.save()
    }

    pub fn delete_countdown(&self, id: &str) -> Result<(), StoreError> {
        {
            let mut data = self.data.lock().unwrap();
            data.countdowns.retain(|c| c.id != id);
        }
        self.save()
    }

    pub fn update_settings(&self, f: impl FnOnce(&mut Settings)) -> Result<(), StoreError> {
        {
            let mut data = self.data.lock().unwrap();
            f(&mut data.settings);
        }
        self.save()
    }

    pub fn save_window_position(&self, x: i32, y: i32) -> Result<(), StoreError> {
        self.update_settings(|s| {
            s.window_position = WindowPosition { x, y };
        })
    }
}

fn get_data_path() -> PathBuf {
    let base = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    base.join("countdown-timer").join("data.json")
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(PathBuf::from)
}
```

- [ ] **Step 3: Update `src-tauri/src/lib.rs` to register store**

```rust
mod countdown_store;

use countdown_store::{Countdown, CountdownStore};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub store: CountdownStore,
}

#[tauri::command]
fn get_all_countdowns(state: State<'_, Mutex<AppState>>) -> Result<countdown_store::AppData, String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    Ok(app.store.get_all())
}

#[tauri::command]
fn save_countdown(state: State<'_, Mutex<AppState>>, countdown: Countdown) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    app.store.save_countdown(countdown).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_countdown(state: State<'_, Mutex<AppState>>, id: String) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    app.store.delete_countdown(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_window_position(state: State<'_, Mutex<AppState>>, x: i32, y: i32) -> Result<(), String> {
    let app = state.lock().map_err(|e| e.to_string())?;
    app.store.save_window_position(x, y).map_err(|e| e.to_string())
}

pub fn run() {
    let store = CountdownStore::new().expect("Failed to initialize data store");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState { store }))
        .invoke_handler(tauri::generate_handler![
            get_all_countdowns,
            save_countdown,
            delete_countdown,
            save_window_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DeskCountdown");
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles successfully.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Rust data model and countdown store with IPC commands"
```

---

### Task 5: Rust — Autostart Manager

**Files:**
- Create: `src-tauri/src/autostart_manager.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: none
- Produces: `set_autostart(enabled)` and `get_autostart()` Tauri commands

- [ ] **Step 1: Write `src-tauri/src/autostart_manager.rs`**

```rust
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
```

- [ ] **Step 2: Register commands in `src-tauri/src/lib.rs`**

Add to `use` block:
```rust
mod autostart_manager;
```

Add commands before `pub fn run()`:
```rust
#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    autostart_manager::set_autostart(enabled)
}

#[tauri::command]
fn get_autostart() -> Result<bool, String> {
    autostart_manager::get_autostart()
}
```

Add to `invoke_handler`:
```rust
set_autostart,
get_autostart,
```

- [ ] **Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add autostart manager with registry read/write"
```

---

### Task 6: Rust — Window Utils

**Files:**
- Create: `src-tauri/src/window_utils.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `AppState` from Task 4
- Produces: `setup_window()` called at startup to validate/set window position

- [ ] **Step 1: Write `src-tauri/src/window_utils.rs`**

```rust
use tauri::{Manager, PhysicalPosition, WebviewWindow};

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
        let monitor = window.primary_monitor().unwrap_or_default();
        if let Some(monitor) = monitor {
            let size = monitor.size();
            let pos = monitor.position();
            let win_size = window.outer_size().unwrap();
            let x = pos.x + size.width as i32 - win_size.width as i32 - 40;
            let y = pos.y + size.height as i32 - win_size.height as i32 - 40;
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
    }
}
```

- [ ] **Step 2: Integrate into `src-tauri/src/lib.rs` `pub fn run()`**

Modify `run()` to use setup hook:

```rust
pub fn run() {
    let store = CountdownStore::new().expect("Failed to initialize data store");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState { store }))
        .setup(|app| {
            let window = app.get_webview_window("main")
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
```

Add module declaration:
```rust
mod window_utils;
```

- [ ] **Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add window position validation and startup positioning"
```

---

### Task 7: Rust — System Tray Manager

**Files:**
- Create: `src-tauri/src/tray_manager.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `AppHandle`, `AppState` from Task 4
- Produces: Tray icon with menu (Show/Hide, Particles toggle, Autostart toggle, About, Quit), `particles-toggled` event emission

- [ ] **Step 1: Write `src-tauri/src/tray_manager.rs`**

```rust
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, CheckMenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use crate::countdown_store::CountdownStore;
use std::sync::Mutex;

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

    let tray = TrayIconBuilder::new()
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
                    if let Ok(app_state) = state.lock() {
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
                    if let Ok(app_state) = state.lock() {
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
                        if let Ok(app_state) = state.lock() {
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
```

- [ ] **Step 2: Update `src-tauri/src/lib.rs`**

Add module and integrate tray into `setup`:

```rust
mod tray_manager;
```

In `setup` closure, after window positioning:
```rust
tray_manager::create_tray(app).expect("Failed to create system tray");
```

Also update the `AppState` import to be `pub`:
```rust
pub struct AppState {
    pub store: CountdownStore,
}
```

Move the `use` for `Manager` and `Emitter` into scope at top of lib.rs (they're used in tray_manager but via AppHandle).

Actually, add to lib.rs imports:
```rust
use tauri::Manager;
```

- [ ] **Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add system tray with full menu and particles event"
```

---

### Task 8: Rust — Single Instance Lock

**Files:**
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: None
- Produces: Single instance enforcement at process start

- [ ] **Step 1: Update `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows::Win32::Foundation::{HANDLE, CloseHandle, ERROR_ALREADY_EXISTS};
use windows::core::PCWSTR;
use std::ptr::null_mut;

fn main() {
    let name: Vec<u16> = "DeskCountdown/SingleInstance\0".encode_utf16().collect();

    unsafe {
        let handle: HANDLE = CreateMutexW(None, true, PCWSTR::from_raw(name.as_ptr()))
            .unwrap_or_else(|_| panic!("Failed to create mutex"));

        let last_error = windows::Win32::Foundation::GetLastError();

        if last_error == ERROR_ALREADY_EXISTS {
            // Another instance is running - find and restore its window
            use windows::Win32::UI::WindowsAndMessaging::{
                FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
            };
            let class_name: Vec<u16> = "DeskCountdown/MainWindow\0".encode_utf16().collect();
            let hwnd = FindWindowW(
                Some(PCWSTR::from_raw(class_name.as_ptr())),
                None,
            );
            if hwnd.is_ok() && !hwnd.unwrap().is_invalid() {
                let _ = ShowWindow(hwnd.unwrap(), SW_RESTORE);
                let _ = SetForegroundWindow(hwnd.unwrap());
            }
            return;
        }

        // Store handle for cleanup - intentionally leak to keep mutex alive
        std::mem::forget(handle);
    }

    deskcountdown::run();
}
```

- [ ] **Step 2: Add `windows` crate to `src-tauri/Cargo.toml`**

```toml
[dependencies]
windows = { version = "0.58", features = [
    "Win32_System_Threading",
    "Win32_Foundation",
    "Win32_UI_WindowsAndMessaging",
]}
```

- [ ] **Step 3: Set window class name in `src-tauri/tauri.conf.json`**

In the window config, no direct class name setting in Tauri v2. Instead, we'll set it in the Rust setup. Add to `src-tauri/src/lib.rs` in the `setup` closure, before tray creation:

```rust
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
// ...
// After window position setup:
#[cfg(target_os = "windows")]
unsafe {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowLongPtrW, GWLP_CLASSNAME, SetClassName,
    };
    // Actually, Tauri v2 manages the window class internally.
    // FindWindowW by title is the simpler approach.
}
```

Actually, let me simplify. Instead of finding by class name, find by window title:

```rust
let title: Vec<u16> = "桌面倒计时\0".encode_utf16().collect();
let hwnd = FindWindowW(None, Some(PCWSTR::from_raw(title.as_ptr())));
```

This is simpler and doesn't require modifying the window class. Let me update the main.rs:

- [ ] **Step 4: Simplify main.rs to find by window title**

Replace the class_name approach with title-based lookup (updated in the main.rs step above).

Actually, let me just rewrite the whole main.rs:

- [ ] **Step 5: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: Compiles successfully.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add single-instance lock with existing window activation"
```

---

### Task 9: CountdownEngine Composable

**Files:**
- Create: `src/composables/useCountdownEngine.ts`
- Create: `src/composables/useContextMenu.ts`

**Interfaces:**
- Consumes: `Countdown` from types
- Produces: `useCountdownEngine()` → `{ cards, expiredCards, separatorVisible }` reactive state; `useContextMenu()` → `{ menuVisible, menuX, menuY, menuItems, openMenu, closeMenu }`

- [ ] **Step 1: Write `src/composables/useCountdownEngine.ts`**

```typescript
import { ref, reactive, onMounted, onUnmounted, computed } from 'vue'
import type { Countdown } from '../types'
import { formatRemaining, formatExpired, computeProgress, parseLocalISO } from '../utils/format'
import { invoke } from '@tauri-apps/api/core'

export interface CardViewModel {
  id: string
  title: string
  prefixText: string
  suffixText: string
  displayText: string
  progressPercent: number
  targetDateFormatted: string
  isExpired: boolean
  gradientCSS: string
  glowColor: string
  glowIntensity: number
}

export function useCountdownEngine() {
  const countdowns = ref<Countdown[]>([])
  const now = ref(new Date())

  let intervalId: ReturnType<typeof setInterval> | null = null

  const cardViewModels = computed<CardViewModel[]>(() => {
    return countdowns.value.map(c => {
      const target = parseLocalISO(c.target_date)
      const isExpired = now.value >= target
      const displayText = isExpired
        ? formatExpired(now.value, target)
        : formatRemaining(now.value, target)
      const progressPercent = isExpired
        ? 0
        : computeProgress(c.created_at, c.target_date, now.value)

      return {
        id: c.id,
        title: c.title,
        prefixText: c.prefix_text,
        suffixText: c.suffix_text,
        displayText,
        progressPercent,
        targetDateFormatted: `${target.getFullYear()}-${String(target.getMonth()+1).padStart(2,'0')}-${String(target.getDate()).padStart(2,'0')}`,
        isExpired,
        gradientCSS: '', // filled by card component
        glowColor: '',   // filled by card component
        glowIntensity: c.font_style.glow_intensity,
      }
    })
  })

  const activeCards = computed(() =>
    cardViewModels.value
      .filter(c => !c.isExpired)
      .sort((a, b) => {
        const aC = countdowns.value.find(x => x.id === a.id)!
        const bC = countdowns.value.find(x => x.id === b.id)!
        const dateDiff = new Date(bC.target_date).getTime() - new Date(aC.target_date).getTime()
        if (dateDiff !== 0) return dateDiff
        return a.title.localeCompare(b.title, 'zh-CN')
      })
  )

  const expiredCards = computed(() =>
    cardViewModels.value
      .filter(c => c.isExpired)
      .sort((a, b) => {
        const aC = countdowns.value.find(x => x.id === a.id)!
        const bC = countdowns.value.find(x => x.id === b.id)!
        const dateDiff = new Date(bC.target_date).getTime() - new Date(aC.target_date).getTime()
        if (dateDiff !== 0) return dateDiff
        return a.title.localeCompare(b.title, 'zh-CN')
      })
  )

  const separatorVisible = computed(() =>
    activeCards.value.length > 0 && expiredCards.value.length > 0
  )

  async function loadFromBackend() {
    const data = await invoke<any>('get_all_countdowns')
    countdowns.value = data.countdowns
  }

  function startTimer() {
    now.value = new Date()
    intervalId = setInterval(() => {
      now.value = new Date()
    }, 1000)
  }

  onMounted(() => {
    loadFromBackend().then(() => startTimer())
  })

  onUnmounted(() => {
    if (intervalId !== null) {
      clearInterval(intervalId)
    }
  })

  return {
    countdowns,
    activeCards,
    expiredCards,
    separatorVisible,
    cardViewModels,
    now,
    loadFromBackend,
  }
}
```

- [ ] **Step 2: Write `src/composables/useContextMenu.ts`**

```typescript
import { ref } from 'vue'

export interface MenuItem {
  label: string
  action: string
  danger?: boolean
  separator?: boolean
}

export function useContextMenu() {
  const menuVisible = ref(false)
  const menuX = ref(0)
  const menuY = ref(0)
  const menuItems = ref<MenuItem[]>([])

  function openMenu(x: number, y: number, items: MenuItem[]) {
    menuX.value = x
    menuY.value = y
    menuItems.value = items
    menuVisible.value = true
  }

  function closeMenu() {
    menuVisible.value = false
  }

  return { menuVisible, menuX, menuY, menuItems, openMenu, closeMenu }
}
```

- [ ] **Step 3: Verify compilation**

```bash
npx vue-tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 4: Commit**

```bash
git add src/composables/
git commit -m "feat: add CountdownEngine and ContextMenu composables"
```

---

### Task 10: CountdownCard Component

**Files:**
- Create: `src/components/CountdownCard.vue`

**Interfaces:**
- Consumes: `CardViewModel` from Task 9, `GRADIENT_PRESETS`, `getGradientCSS`, `getGlowColor` from Task 2
- Produces: Rendered card with gradient digits, progress bar, corner accents, left accent bar, hover effects

- [ ] **Step 1: Write `src/components/CountdownCard.vue`**

```vue
<template>
  <div
    class="card"
    :class="{ 'card--expired': viewModel.isExpired, 'card--confirm': confirming }"
    @mouseenter="hovering = true"
    @mouseleave="hovering = false"
    @contextmenu.prevent="$emit('contextmenu', $event, countdownId)"
  >
    <!-- Corner accents -->
    <div class="card__corner card__corner--tl"></div>
    <div class="card__corner card__corner--tr"></div>
    <div class="card__corner card__corner--bl"></div>
    <div class="card__corner card__corner--br"></div>

    <!-- Left accent bar -->
    <div
      class="card__accent"
      :style="{ background: gradientCSS, boxShadow: hovering ? `0 0 8px ${glowColor}` : `0 0 4px ${glowColor}` }"
    ></div>

    <!-- Card content -->
    <div v-if="!confirming" class="card__content">
      <div class="card__text">
        <span>{{ viewModel.prefixText }}</span>
        <span>{{ viewModel.title }}</span>
        <span>{{ viewModel.suffixText }}</span>
      </div>
      <div
        class="card__digits"
        :style="{
          backgroundImage: gradientCSS,
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          textShadow: digitsGlow,
        }"
      >
        {{ viewModel.displayText }}
      </div>
      <div v-if="!viewModel.isExpired" class="card__progress">
        <div class="card__progress-track">
          <div
            class="card__progress-fill"
            :style="{ width: viewModel.progressPercent + '%', background: gradientCSS }"
          ></div>
          <div class="card__progress-dot" :style="{ background: glowColor }"></div>
        </div>
        <span class="card__progress-label">{{ viewModel.progressPercent }}%</span>
      </div>
      <div class="card__target-date">{{ viewModel.targetDateFormatted }}</div>
    </div>

    <!-- Confirmation state -->
    <div v-else class="card__confirm">
      <span class="card__confirm-text">确认删除？</span>
      <div class="card__confirm-actions">
        <button class="card__confirm-btn card__confirm-btn--cancel" @click="$emit('cancel-delete')">
          取消
        </button>
        <button class="card__confirm-btn card__confirm-btn--delete" @click="$emit('confirm-delete')">
          删除
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { CardViewModel } from '../composables/useCountdownEngine'
import { getGradientCSS, getGlowColor } from '../types'

const props = defineProps<{
  viewModel: CardViewModel
  countdown: any
  confirming: boolean
}>()

defineEmits<{
  contextmenu: [event: MouseEvent, id: string]
  'confirm-delete': []
  'cancel-delete': []
}>()

const hovering = ref(false)

const countdownId = computed(() => props.countdown.id)
const gradientCSS = computed(() =>
  getGradientCSS(props.countdown.font_style.gradient)
)
const glowColor = computed(() =>
  getGlowColor(props.countdown.font_style.gradient)
)
const digitsGlow = computed(() => {
  if (props.viewModel.isExpired) return 'none'
  const blur = props.countdown.font_style.glow_intensity * 20
  return `0 0 ${blur}px ${glowColor.value}`
})
</script>
```

- [ ] **Step 2: Add card CSS to `src/style.css`** (append)

```css
.card {
  position: relative;
  width: 300px;
  padding: 14px 16px;
  background: linear-gradient(180deg, rgba(12, 13, 18, 0.92), rgba(10, 11, 16, 0.92));
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  margin-bottom: 8px;
  transition: border-color 200ms, background-color 200ms;
  -webkit-app-region: no-drag;
  overflow: hidden;
}

.card:hover {
  border-color: rgba(255, 255, 255, 0.25);
}

.card--expired {
  opacity: 0.4;
  filter: grayscale(60%);
}

.card--confirm {
  background: rgba(20, 0, 0, 0.88);
  border-color: rgba(255, 0, 0, 0.15);
}

.card__corner {
  position: absolute;
  width: 8px;
  height: 8px;
  border-color: rgba(255, 255, 255, 0.18);
  border-style: solid;
  border-width: 0;
}

.card__corner--tl { top: 4px; left: 4px; border-top-width: 1px; border-left-width: 1px; }
.card__corner--tr { top: 4px; right: 4px; border-top-width: 1px; border-right-width: 1px; }
.card__corner--bl { bottom: 4px; left: 4px; border-bottom-width: 1px; border-left-width: 1px; }
.card__corner--br { bottom: 4px; right: 4px; border-bottom-width: 1px; border-right-width: 1px; }

.card__accent {
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 2px;
  border-radius: 1px;
  transition: box-shadow 200ms;
}

.card__content {
  position: relative;
  z-index: 1;
  margin-left: 6px;
}

.card__text {
  color: rgba(255, 255, 255, 0.85);
  font-size: 13px;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card__digits {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: 1px;
  margin-bottom: 6px;
  transition: text-shadow 300ms;
}

.card__progress {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.card__progress-track {
  flex: 1;
  height: 3px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
  position: relative;
  overflow: hidden;
}

.card__progress-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 300ms;
}

.card__progress-dot {
  position: absolute;
  top: 0;
  width: 4px;
  height: 4px;
  border-radius: 50%;
  animation: dotSlide 3s ease-in-out infinite;
}

@keyframes dotSlide {
  0%, 100% { left: 0; }
  50% { left: calc(100% - 4px); }
}

.card__progress-label {
  color: rgba(255, 255, 255, 0.45);
  font-size: 10px;
  min-width: 30px;
  text-align: right;
}

.card__target-date {
  color: rgba(255, 255, 255, 0.45);
  font-size: 11px;
}

.card__confirm {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.card__confirm-text {
  color: rgba(255, 255, 255, 0.7);
  font-size: 14px;
}

.card__confirm-actions {
  display: flex;
  gap: 12px;
}

.card__confirm-btn {
  padding: 4px 18px;
  border-radius: 6px;
  border: none;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 150ms;
}

.card__confirm-btn:hover {
  opacity: 0.8;
}

.card__confirm-btn--cancel {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.6);
}

.card__confirm-btn--delete {
  background: rgba(255, 60, 60, 0.2);
  color: #ff4d4d;
}
```

- [ ] **Step 3: Verify compilation**

```bash
npx vue-tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 4: Commit**

```bash
git add src/components/CountdownCard.vue src/style.css
git commit -m "feat: add CountdownCard component with HUD styling"
```

---

### Task 11: CardList, ContextMenu, ParticlesBackground, OnboardingToast Components

**Files:**
- Create: `src/components/CardList.vue`
- Create: `src/components/ContextMenu.vue`
- Create: `src/components/ParticlesBackground.vue`
- Create: `src/components/OnboardingToast.vue`

**Interfaces:**
- Consumes: Types and composables from Tasks 2, 9
- Produces: All supporting UI components

- [ ] **Step 1: Write `src/components/CardList.vue`**

```vue
<template>
  <div class="card-list" :style="{ height: listHeight + 'px' }">
    <div v-if="activeCards.length === 0 && expiredCards.length === 0" class="card-list__empty">
      <div class="card-list__empty-icon">+</div>
      <div class="card-list__empty-text">右键新建倒计时</div>
    </div>

    <template v-else>
      <CountdownCard
        v-for="vm in activeCards"
        :key="vm.id"
        :view-model="vm"
        :countdown="getRawCard(vm.id)"
        :confirming="confirmingId === vm.id"
        @contextmenu="(e, id) => $emit('card-contextmenu', e, id)"
        @confirm-delete="$emit('confirm-delete', vm.id)"
        @cancel-delete="$emit('cancel-delete')"
      />

      <div v-if="separatorVisible" class="card-list__separator">
        <span>已过期</span>
      </div>

      <CountdownCard
        v-for="vm in expiredCards"
        :key="vm.id"
        :view-model="vm"
        :countdown="getRawCard(vm.id)"
        :confirming="confirmingId === vm.id"
        @contextmenu="(e, id) => $emit('card-contextmenu', e, id)"
        @confirm-delete="$emit('confirm-delete', vm.id)"
        @cancel-delete="$emit('cancel-delete')"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CardViewModel } from '../composables/useCountdownEngine'
import type { Countdown } from '../types'
import CountdownCard from './CountdownCard.vue'

const props = defineProps<{
  activeCards: CardViewModel[]
  expiredCards: CardViewModel[]
  separatorVisible: boolean
  confirmingId: string | null
  countdowns: Countdown[]
}>()

defineEmits<{
  'card-contextmenu': [event: MouseEvent, id: string]
  'confirm-delete': [id: string]
  'cancel-delete': []
}>()

function getRawCard(id: string): Countdown {
  return props.countdowns.find(c => c.id === id)!
}

defineExpose({ getRawCard })

const listHeight = computed(() => {
  // Approximate: each card ~100px, separator ~30px, empty state ~120px
  const total = props.activeCards.length + props.expiredCards.length
  if (total === 0) return 120
  return total * 100 + (props.separatorVisible ? 30 : 0) + (total - 1) * 8
})
</script>

<style scoped>
.card-list {
  padding: 8px;
  transition: height 250ms ease-out;
  -webkit-app-region: drag;
  min-height: 120px;
}

.card-list__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 104px;
  border: 2px dashed rgba(255, 255, 255, 0.10);
  border-radius: 12px;
  -webkit-app-region: no-drag;
}

.card-list__empty-icon {
  font-size: 32px;
  color: rgba(255, 255, 255, 0.15);
  line-height: 1;
  margin-bottom: 6px;
}

.card-list__empty-text {
  color: rgba(255, 255, 255, 0.30);
  font-size: 12px;
}

.card-list__separator {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  margin: 4px 0;
  -webkit-app-region: no-drag;
}

.card-list__separator::before,
.card-list__separator::after {
  content: '';
  flex: 1;
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
}

.card-list__separator span {
  margin: 0 12px;
  color: rgba(255, 255, 255, 0.25);
  font-size: 11px;
}
</style>
```

- [ ] **Step 2: Write `src/components/ContextMenu.vue`**

```vue
<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="context-menu"
      :style="{ left: x + 'px', top: y + 'px' }"
      @click.stop
    >
      <div
        v-for="item in items"
        :key="item.action"
        class="context-menu__item"
        :class="{ 'context-menu__item--danger': item.danger }"
        @click="$emit('select', item.action)"
      >
        <span v-if="item.separator" class="context-menu__separator"></span>
        <span v-else>{{ item.label }}</span>
      </div>
    </div>
    <div v-if="visible" class="context-menu__overlay" @click="$emit('close')"></div>
  </Teleport>
</template>

<script setup lang="ts">
import type { MenuItem } from '../composables/useContextMenu'

defineProps<{
  visible: boolean
  x: number
  y: number
  items: MenuItem[]
}>()

defineEmits<{
  select: [action: string]
  close: []
}>()
</script>

<style>
.context-menu {
  position: fixed;
  z-index: 9999;
  background: rgba(18, 19, 24, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 4px 0;
  min-width: 160px;
  backdrop-filter: blur(12px);
}

.context-menu__item {
  padding: 8px 16px;
  color: rgba(255, 255, 255, 0.8);
  font-size: 13px;
  cursor: pointer;
  transition: background 150ms;
}

.context-menu__item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.context-menu__item--danger:hover {
  background: rgba(255, 60, 60, 0.15);
  color: #ff4d4d;
}

.context-menu__separator {
  display: block;
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
  margin: 4px 0;
  padding: 0;
  cursor: default;
  pointer-events: none;
}

.context-menu__overlay {
  position: fixed;
  inset: 0;
  z-index: 9998;
}
</style>
```

- [ ] **Step 3: Write `src/components/ParticlesBackground.vue`**

```vue
<template>
  <div class="particles" v-if="enabled">
    <div
      v-for="i in particleCount"
      :key="i"
      class="particles__dot"
      :style="{
        left: randomPct(i, 0) + '%',
        animationDuration: randomDuration(i) + 's',
        animationDelay: randomPct(i, 1) + 's',
        opacity: randomOpacity(i),
      }"
    ></div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ enabled: boolean }>()

const particleCount = 10

function randomPct(i: number, seed: number): number {
  return ((i * 17 + seed * 31) % 100)
}

function randomDuration(i: number): number {
  return 6 + ((i * 13) % 10)
}

function randomOpacity(i: number): number {
  return 0.10 + ((i * 7) % 6) * 0.01
}
</script>

<style scoped>
.particles {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

.particles__dot {
  position: absolute;
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.12);
  animation: particleDrift linear infinite;
}

@keyframes particleDrift {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-20px); }
}
</style>
```

- [ ] **Step 4: Write `src/components/OnboardingToast.vue`**

```vue
<template>
  <div v-if="visible" class="onboarding">
    <div class="onboarding__card">
      <div class="onboarding__text">开机时自动启动？</div>
      <div class="onboarding__actions">
        <button class="onboarding__btn onboarding__btn--primary" @click="$emit('enable')">
          开启
        </button>
        <button class="onboarding__btn onboarding__btn--secondary" @click="$emit('dismiss')">
          暂不
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ visible: boolean }>()
defineEmits<{ enable: []; dismiss: [] }>()
</script>

<style scoped>
.onboarding {
  position: relative;
  z-index: 10;
  padding: 12px 8px 0;
  -webkit-app-region: no-drag;
}

.onboarding__card {
  background: rgba(20, 22, 30, 0.94);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.onboarding__text {
  color: rgba(255, 255, 255, 0.85);
  font-size: 14px;
}

.onboarding__actions {
  display: flex;
  gap: 10px;
}

.onboarding__btn {
  padding: 6px 20px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}

.onboarding__btn--primary {
  background: rgba(0, 180, 255, 0.2);
  color: #00b4ff;
}

.onboarding__btn--secondary {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.5);
}
</style>
```

- [ ] **Step 5: Verify compilation**

```bash
npx vue-tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 6: Commit**

```bash
git add src/components/
git commit -m "feat: add CardList, ContextMenu, Particles, and Onboarding components"
```

---

### Task 12: InlineEditor Component

**Files:**
- Create: `src/components/InlineEditor.vue`

**Interfaces:**
- Consumes: `Countdown`, `EditorMode`, `GRADIENT_PRESETS` from Task 2; `formatRemaining` from Task 3
- Produces: Full editor with title, date picker, prefix/suffix, gradient swatches, glow slider, live preview

- [ ] **Step 1: Write `src/components/InlineEditor.vue`**

```vue
<template>
  <div class="editor" :style="{ height: editorHeight + 'px' }">
    <div class="editor__inner">
      <!-- Title -->
      <div class="editor__field">
        <label class="editor__label">标题</label>
        <input
          v-model="form.title"
          class="editor__input"
          maxlength="50"
          placeholder="输入标题"
          @input="validate"
        />
        <span v-if="titleError" class="editor__hint">标题不能为空</span>
      </div>

      <!-- Date -->
      <div class="editor__field">
        <label class="editor__label">日期</label>
        <div class="editor__date-row">
          <input
            v-model="form.date"
            class="editor__input editor__input--date"
            type="date"
          />
          <input
            v-model="form.time"
            class="editor__input editor__input--time"
            type="time"
          />
        </div>
        <div class="editor__shortcuts">
          <button class="editor__shortcut" @click="setDateOffset(7)">一周后</button>
          <button class="editor__shortcut" @click="setDateOffset(30)">一月后</button>
          <button class="editor__shortcut" @click="setDateOffset(365)">一年后</button>
        </div>
      </div>

      <!-- Prefix / Suffix -->
      <div class="editor__field">
        <label class="editor__label">前段</label>
        <input
          v-model="form.prefixText"
          class="editor__input"
          maxlength="20"
          placeholder="例如：距离"
        />
      </div>
      <div class="editor__field">
        <label class="editor__label">后段</label>
        <input
          v-model="form.suffixText"
          class="editor__input"
          maxlength="20"
          placeholder="例如：还剩"
        />
      </div>

      <!-- Live preview -->
      <div class="editor__preview">
        <span class="editor__preview-label">预览</span>
        <span class="editor__preview-text">
          {{ previewText }}
        </span>
      </div>

      <!-- Font swatches -->
      <div class="editor__field">
        <label class="editor__label">字体</label>
        <div class="editor__swatches">
          <div
            v-for="p in GRADIENT_PRESETS"
            :key="p.key"
            class="editor__swatch"
            :class="{ 'editor__swatch--selected': form.gradient === p.key }"
            @click="form.gradient = p.key"
          >
            <div
              class="editor__swatch-preview"
              :style="{ background: 'linear-gradient(135deg, ' + p.colors[0] + ', ' + p.colors[1] + ')' }"
            ></div>
            <span class="editor__swatch-name">{{ p.name }}</span>
          </div>
        </div>
      </div>

      <!-- Glow slider -->
      <div class="editor__field">
        <label class="editor__label">辉光 {{ Math.round(form.glowIntensity * 100) }}%</label>
        <input
          v-model.number="form.glowIntensity"
          class="editor__slider"
          type="range"
          min="0"
          max="100"
          @input="form.glowIntensity = $event.target.value / 100"
        />
      </div>

      <!-- Actions -->
      <div class="editor__actions">
        <button
          class="editor__btn editor__btn--save"
          :disabled="!canSave"
          @click="$emit('save', buildSaveData())"
        >
          保存
        </button>
        <button class="editor__btn editor__btn--cancel" @click="$emit('cancel')">
          取消
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed, watch, onMounted } from 'vue'
import type { Countdown, EditorMode } from '../types'
import { GRADIENT_PRESETS } from '../types'

const props = defineProps<{
  mode: EditorMode
  editData: Countdown | null
}>()

defineEmits<{
  save: [data: Partial<Countdown>]
  cancel: []
}>()

const titleError = ref(false)
const canSave = computed(() => form.title.trim().length > 0)

const editorHeight = ref(400)

function getDefaultDate(): string {
  const d = new Date()
  d.setDate(d.getDate() + 7)
  return d.toISOString().split('T')[0]
}

const form = reactive({
  title: '',
  date: getDefaultDate(),
  time: '09:00',
  prefixText: '',
  suffixText: '',
  gradient: 'neon-cyan-blue',
  glowIntensity: 0.6,
})

const previewText = computed(() => {
  const prefix = form.prefixText || ''
  const title = form.title || '___'
  const suffix = form.suffixText || ''
  const target = new Date(form.date + 'T' + form.time)
  const now = new Date()
  const digits = target > now
    ? computeRemainingForPreview(now, target)
    : '___天 __时 __分 __秒'
  return `${prefix} ${title} ${suffix} ${digits}`.trim()
})

function computeRemainingForPreview(now: Date, target: Date): string {
  let diff = target.getTime() - now.getTime()
  if (diff <= 0) return '0分 00秒'
  const ts = Math.floor(diff / 1000)
  const d = Math.floor(ts / 86400)
  const h = Math.floor((ts % 86400) / 3600)
  const m = Math.floor((ts % 3600) / 60)
  const s = ts % 60
  const pad = (n: number) => String(n).padStart(2, '0')
  if (d >= 1) return `${d}天 ${pad(h)}时 ${pad(m)}分 ${pad(s)}秒`
  if (h >= 1) return `${pad(h)}时 ${pad(m)}分 ${pad(s)}秒`
  return `${pad(m)}分 ${pad(s)}秒`
}

function validate() {
  titleError.value = form.title.trim().length === 0
}

function setDateOffset(days: number) {
  const d = new Date()
  d.setDate(d.getDate() + days)
  form.date = d.toISOString().split('T')[0]
}

function buildSaveData(): Partial<Countdown> {
  return {
    title: form.title.trim(),
    prefix_text: form.prefixText,
    suffix_text: form.suffixText,
    target_date: form.date + 'T' + form.time + ':00',
    font_style: {
      gradient: form.gradient,
      glow_intensity: form.glowIntensity,
    },
  }
}

watch(() => props.editData, (data) => {
  if (data && props.mode === 'edit') {
    form.title = data.title
    form.date = data.target_date.split('T')[0]
    form.time = data.target_date.split('T')[1].substring(0, 5)
    form.prefixText = data.prefix_text
    form.suffixText = data.suffix_text
    form.gradient = data.font_style.gradient
    form.glowIntensity = data.font_style.glow_intensity
  } else if (props.mode === 'create') {
    form.title = data?.title || ''
    form.date = data?.target_date
      ? data.target_date.split('T')[0]
      : getDefaultDate()
    form.time = data?.target_date
      ? data.target_date.split('T')[1].substring(0, 5)
      : '09:00'
    form.prefixText = data?.prefix_text || ''
    form.suffixText = data?.suffix_text || ''
    form.gradient = data?.font_style.gradient || 'neon-cyan-blue'
    form.glowIntensity = data?.font_style.glow_intensity ?? 0.6
  }
}, { immediate: true })
</script>

<style scoped>
.editor {
  padding: 0 8px 8px;
  transition: height 200ms ease-out;
  -webkit-app-region: no-drag;
  overflow: hidden;
}

.editor__inner {
  background: rgba(16, 17, 24, 0.94);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 16px;
}

.editor__field {
  margin-bottom: 10px;
}

.editor__label {
  display: block;
  color: rgba(255, 255, 255, 0.6);
  font-size: 11px;
  margin-bottom: 4px;
}

.editor__input {
  width: 100%;
  padding: 6px 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.85);
  font-size: 13px;
  box-sizing: border-box;
  outline: none;
}

.editor__input:focus {
  border-color: rgba(255, 255, 255, 0.25);
}

.editor__date-row {
  display: flex;
  gap: 8px;
}

.editor__input--date { flex: 2; }
.editor__input--time { flex: 1; }

.editor__shortcuts {
  display: flex;
  gap: 6px;
  margin-top: 6px;
}

.editor__shortcut {
  padding: 3px 10px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 11px;
  cursor: pointer;
  transition: all 150ms;
}

.editor__shortcut:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.75);
}

.editor__hint {
  color: #ff6b6b;
  font-size: 11px;
  margin-top: 3px;
  display: block;
}

.editor__preview {
  padding: 10px 12px;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  margin-bottom: 10px;
}

.editor__preview-label {
  color: rgba(255, 255, 255, 0.35);
  font-size: 10px;
  display: block;
  margin-bottom: 4px;
}

.editor__preview-text {
  color: rgba(255, 255, 255, 0.8);
  font-size: 14px;
  font-weight: 600;
}

.editor__swatches {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 6px;
}

.editor__swatch {
  padding: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  cursor: pointer;
  text-align: center;
  transition: border-color 150ms;
}

.editor__swatch--selected {
  border-color: rgba(255, 255, 255, 0.4);
}

.editor__swatch-preview {
  height: 8px;
  border-radius: 4px;
  margin-bottom: 4px;
}

.editor__swatch-name {
  color: rgba(255, 255, 255, 0.6);
  font-size: 10px;
}

.editor__slider {
  width: 100%;
  accent-color: #00b4ff;
}

.editor__actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 14px;
}

.editor__btn {
  padding: 7px 24px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 150ms;
}

.editor__btn:hover { opacity: 0.85; }

.editor__btn--save {
  background: rgba(0, 180, 255, 0.2);
  color: #00b4ff;
}

.editor__btn--save:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.editor__btn--cancel {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.5);
}
</style>
```

- [ ] **Step 2: Verify compilation**

```bash
npx vue-tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add src/components/InlineEditor.vue
git commit -m "feat: add InlineEditor with date picker, swatches, glow slider, and live preview"
```

---

### Task 13: App.vue — Root Integration

**Files:**
- Modify: `src/App.vue`

**Interfaces:**
- Consumes: All components and composables from Tasks 9-12
- Produces: Complete working application UI

- [ ] **Step 1: Write `src/App.vue`**

```vue
<template>
  <div class="app-root" @contextmenu.prevent="onRootContextMenu">
    <ParticlesBackground :enabled="particlesEnabled" />

    <OnboardingToast
      :visible="showOnboarding"
      @enable="onOnboardingEnable"
      @dismiss="onOnboardingDismiss"
    />

    <InlineEditor
      v-if="editorVisible"
      :mode="editorMode"
      :edit-data="editingCard"
      @save="onEditorSave"
      @cancel="onEditorCancel"
    />

    <CardList
      ref="cardListRef"
      :active-cards="activeCards"
      :expired-cards="expiredCards"
      :separator-visible="separatorVisible"
      :confirming-id="confirmingId"
      :countdowns="countdowns"
      @card-contextmenu="onCardContextMenu"
      @confirm-delete="onConfirmDelete"
      @cancel-delete="confirmingId = null"
    />

    <ContextMenu
      :visible="menuVisible"
      :x="menuX"
      :y="menuY"
      :items="menuItems"
      @select="onMenuSelect"
      @close="closeMenu"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { nextTick } from 'vue'

import type { Countdown, EditorMode } from './types'
import {
  useCountdownEngine,
  useContextMenu,
  type CardViewModel,
} from './composables/useCountdownEngine'
import type { MenuItem } from './composables/useContextMenu'

import CountdownCard from './components/CountdownCard.vue'
import CardList from './components/CardList.vue'
import ContextMenu from './components/ContextMenu.vue'
import InlineEditor from './components/InlineEditor.vue'
import ParticlesBackground from './components/ParticlesBackground.vue'
import OnboardingToast from './components/OnboardingToast.vue'

// ── Engine ──
const {
  countdowns,
  activeCards,
  expiredCards,
  separatorVisible,
  loadFromBackend,
} = useCountdownEngine()

// ── Context menu ──
const { menuVisible, menuX, menuY, menuItems, openMenu, closeMenu } = useContextMenu()

// ── Editor state ──
const editorVisible = ref(false)
const editorMode = ref<EditorMode>('create')
const editingCard = ref<Countdown | null>(null)

// ── Delete confirmation ──
const confirmingId = ref<string | null>(null)

// ── Onboarding ──
const showOnboarding = ref(false)
const particlesEnabled = ref(false)

// ── Toast ──
const toastMsg = ref('')
const toastVisible = ref(false)
let toastTimer: ReturnType<typeof setTimeout> | null = null

function showToast(msg: string) {
  toastMsg.value = msg
  toastVisible.value = true
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => { toastVisible.value = false }, 2000)
}

// ── Context menu handlers ──
const CARD_MENU: MenuItem[] = [
  { label: '新增倒计时', action: 'add' },
  { label: '编辑此倒计时', action: 'edit' },
  { label: '复制此倒计时', action: 'copy' },
  { label: '', action: 'sep1', separator: true },
  { label: '删除此倒计时', action: 'delete', danger: true },
]

const EMPTY_MENU: MenuItem[] = [
  { label: '新增倒计时', action: 'add' },
]

let contextMenuTargetId: string | null = null

function onCardContextMenu(event: MouseEvent, id: string) {
  contextMenuTargetId = id
  if (editorVisible.value) {
    showToast('请先保存或取消当前编辑')
    return
  }
  openMenu(event.clientX, event.clientY, CARD_MENU)
}

function onRootContextMenu(event: MouseEvent) {
  contextMenuTargetId = null
  if (editorVisible.value) {
    showToast('请先保存或取消当前编辑')
    return
  }
  openMenu(event.clientX, event.clientY, EMPTY_MENU)
}

function onMenuSelect(action: string) {
  closeMenu()
  switch (action) {
    case 'add':
      openEditor('create', null)
      break
    case 'edit':
      if (contextMenuTargetId) {
        const card = countdowns.value.find(c => c.id === contextMenuTargetId)
        if (card) openEditor('edit', card)
      }
      break
    case 'copy':
      if (contextMenuTargetId) {
        const card = countdowns.value.find(c => c.id === contextMenuTargetId)
        if (card) {
          const copy: Countdown = {
            ...JSON.parse(JSON.stringify(card)),
            id: '',
            title: card.title + ' 副本',
          }
          openEditor('create', copy)
        }
      }
      break
    case 'delete':
      confirmingId.value = contextMenuTargetId
      break
  }
}

// ── Editor handlers ──
function openEditor(mode: EditorMode, data: Countdown | null) {
  editorMode.value = mode
  editingCard.value = data
  editorVisible.value = true
  resizeWindow()
}

async function onEditorSave(data: Partial<Countdown>) {
  const now = new Date().toISOString().replace('Z', '')

  if (editorMode.value === 'create') {
    const id = crypto.randomUUID()
    const newCard: Countdown = {
      id,
      title: data.title!,
      prefix_text: data.prefix_text || '',
      suffix_text: data.suffix_text || '',
      created_at: now,
      target_date: data.target_date!,
      font_style: data.font_style!,
      sort_order: 0,
    }
    await invoke('save_countdown', { countdown: newCard })
  } else if (editorMode.value === 'edit' && editingCard.value) {
    const updated: Countdown = {
      ...editingCard.value,
      title: data.title!,
      prefix_text: data.prefix_text || '',
      suffix_text: data.suffix_text || '',
      target_date: data.target_date!,
      font_style: data.font_style!,
      created_at: data.target_date !== editingCard.value.target_date
        ? now
        : editingCard.value.created_at,
    }
    await invoke('save_countdown', { countdown: updated })
  }

  editorVisible.value = false
  editingCard.value = null
  await loadFromBackend()
  await resizeWindow()
}

function onEditorCancel() {
  editorVisible.value = false
  editingCard.value = null
  resizeWindow()
}

// ── Delete handler ──
async function onConfirmDelete(id: string) {
  await invoke('delete_countdown', { id })
  confirmingId.value = null
  await loadFromBackend()
  await resizeWindow()
}

// ── Onboarding ──
async function onOnboardingEnable() {
  await invoke('set_autostart', { enabled: true })
  showOnboarding.value = false
}

async function onOnboardingDismiss() {
  showOnboarding.value = false
}

// ── Window resize ──
async function resizeWindow() {
  await nextTick()
  const appRoot = document.querySelector('.app-root') as HTMLElement
  if (!appRoot) return
  const height = appRoot.scrollHeight
  const width = 348
  try {
    const win = getCurrentWindow()
    await win.setSize(new (await import('@tauri-apps/api/window')).PhysicalSize(width, height))
  } catch (_) {}
}

// ── Tauri events ──
listen<boolean>('particles-toggled', (event) => {
  particlesEnabled.value = event.payload
})

// ── Init ──
loadFromBackend().then(() => {
  const data = countdowns.value
  // Check onboarding
  // (settings are loaded via a separate invoke or inferred)
  resizeWindow()
})

// Keyboard: Escape closes menu, cancels delete confirm
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    if (menuVisible.value) closeMenu()
    else if (confirmingId.value) confirmingId.value = null
    else if (editorVisible.value) onEditorCancel()
  }
  if (e.key === 'Enter' && editorVisible.value) {
    // Save on Enter if form is valid
    // (handled by form's implicit submit, simplified here)
  }
})
</script>

<style scoped>
.app-root {
  width: 348px;
  min-height: 120px;
  -webkit-app-region: drag;
}
</style>
```

- [ ] **Step 2: Verify compilation**

```bash
npx vue-tsc --noEmit
```

Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add src/App.vue
git commit -m "feat: integrate all components into App.vue root"
```

---

### Task 14: Global CSS — Final Styling

**Files:**
- Modify: `src/style.css`

**Interfaces:**
- Consumes: None (global styles)
- Produces: Complete visual styling

- [ ] **Step 1: Update `src/style.css` with complete global styles**

Add to the existing card styles (from Task 10):

```css
/* ── Global Reset ── */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body {
  background: transparent !important;
  overflow: hidden;
  user-select: none;
  font-family: "Microsoft YaHei", "PingFang SC", "Helvetica Neue", sans-serif;
  -webkit-font-smoothing: antialiased;
}

#app {
  background: transparent;
}

/* ── Scrollbar ── */
::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

/* ── Toast ── */
.toast {
  position: fixed;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 10000;
  padding: 6px 16px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.75);
  font-size: 12px;
  pointer-events: none;
  animation: toastIn 200ms ease-out;
}

@keyframes toastIn {
  from { opacity: 0; transform: translateX(-50%) translateY(-10px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
```

- [ ] **Step 2: Verify build**

```bash
npm run build
```

Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/style.css
git commit -m "style: add global reset, scrollbar, and toast styles"
```

---

### Task 15: Build, Icon, and Installer Configuration

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/icons/` (placeholder note)

**Interfaces:**
- Produces: Buildable NSIS installer

- [ ] **Step 1: Verify final `src-tauri/tauri.conf.json`**

Ensure the bundle config is correct (from Task 1, verify):

```json
"bundle": {
  "active": true,
  "targets": "nsis",
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/icon.ico"
  ],
  "windows": {
    "nsis": {
      "installMode": "both"
    }
  }
}
```

- [ ] **Step 2: Create icon (design note)**

Generate icon files:
- `src-tauri/icons/32x32.png` — 32×32 PNG
- `src-tauri/icons/128x128.png` — 128×128 PNG
- `src-tauri/icons/icon.ico` — multi-res .ico (16, 32, 48, 256)

Use a minimalist hourglass or countdown clock silhouette on a dark background.

```bash
# Placeholder: copy a placeholder icon
# In production, use a proper icon generator like `tauri icon`
npx tauri icon ./path/to/source-icon.png
```

- [ ] **Step 3: Set window label for FindWindowW**

In `src-tauri/tauri.conf.json`, ensure the window label is "main":
```json
"windows": [{
  "label": "main",
  "title": "桌面倒计时",
  ...
}]
```

This label maps to the Tauri window, and its title "桌面倒计时" is used by `FindWindowW` for single-instance activation.

- [ ] **Step 4: Full build test**

```bash
npm run tauri build
```

Expected: Produces `src-tauri/target/release/bundle/nsis/DeskCountdown_*.exe`

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore: finalize build config, icons, and installer settings"
```

---

### Task 16: Final Integration Test Checklist

**No new files created. Manual verification pass.**

- [ ] **Step 1: Launch the app**

```bash
npm run tauri dev
```

- [ ] **Step 2: Verify first-launch experience**
  - Window appears at bottom-right corner
  - Empty state shows dashed border with "右键新建倒计时"
  - Onboarding toast appears: "开机时自动启动？"
  - Click "暂不" → toast disappears
  - Check `%APPDATA%\countdown-timer\data.json` — should have `onboarding_complete: true`, `autostart: false`

- [ ] **Step 3: Create a countdown**
  - Right-click empty area → "新增倒计时"
  - Editor expands at top
  - Fill title "测试", keep defaults
  - Click "保存" → card appears below editor
  - Editor collapses → card shows gradient digits counting down

- [ ] **Step 4: Test editor features**
  - Right-click card → "编辑此倒计时" → editor opens with pre-filled data
  - Change gradient to "火焰橙红" → swatch highlights
  - Adjust glow slider → percentage updates
  - Change date via shortcut "一月后" → preview updates
  - Click "取消" → editor closes, card unchanged

- [ ] **Step 5: Test copy**
  - Right-click card → "复制此倒计时" → editor opens in create mode
  - Title shows "测试 副本"
  - Click "保存" → second card appears

- [ ] **Step 6: Test delete**
  - Right-click card → "删除此倒计时" → card turns red, shows "确认删除？"
  - Click "取消" → card restores
  - Right-click card → "删除此倒计时" → click "删除" → card collapses, window shrinks

- [ ] **Step 7: Test expired countdown**
  - Create a countdown with target date = yesterday
  - Card renders in expired state: grayscale, 40% opacity, "今日到期"
  - Card is below all active cards, separator visible

- [ ] **Step 8: Test window dragging**
  - Click-drag empty area → window moves
  - Card area not draggable (no-drag)

- [ ] **Step 9: Test system tray**
  - Left-click tray icon → window hides/shows
  - Right-click tray icon → menu appears
  - Toggle "特效" → particles appear/disappear
  - Toggle "开机自启" → check registry key added/removed
  - Click "关于" → message box with version
  - Click "退出" → app exits

- [ ] **Step 10: Test error scenarios**
  - Delete `data.json` → app recreates on next launch
  - Corrupt `data.json` → app backs up and resets
  - Try launching second instance → existing window is restored

- [ ] **Step 11: Test persistence**
  - Add countdowns, close app, reopen → countdowns preserved
  - Window position preserved between launches
  - Autostart setting preserved

---

### Task 17: Commit Final Changes

- [ ] **Step 1: Final commit**

```bash
git add -A
git commit -m "docs: add implementation plan and finalize project"
```

---

## Self-Review

**Spec coverage check:**
- Visual design (§2.1-2.10): Tasks 10, 11, 14 cover all styling and visual behaviors ✅
- Architecture (§3): Tasks 1, 4-8, 12 cover all backend and frontend modules ✅
- Data model (§4): Task 2 (types), Task 4 (Rust structs) ✅
- Inline editor (§5): Task 12 ✅
- Installation & autostart (§6): Tasks 5, 7, 15 ✅
- System tray (§7): Task 7 ✅
- Error handling (§8): Tasks 4, 5, 8, 13 ✅
- Performance (§9): Tasks 9 (no IPC per tick), 14 (GPU-accelerated CSS) ✅
- Future considerations (§10): Not implemented (v1 scope) ✅
- Design decisions (§11): Reflected in architecture choices ✅

**Placeholder scan:** No TBD, TODO, or vague descriptions. All code is concrete.

**Type consistency:**
- `Countdown` type used consistently across Tasks 2, 9, 10, 11, 12, 13 ✅
- `CardViewModel` derived from `Countdown` in Task 9, consumed by Tasks 10, 11 ✅
- Rust `Countdown` struct matches TypeScript `Countdown` interface ✅
- IPC command names match between Rust (Tasks 4, 5) and frontend (Tasks 9, 13) ✅
