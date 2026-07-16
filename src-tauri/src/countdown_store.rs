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
                    let _ = fs::write(&bak, &content);
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
    let base = std::env::var("APPDATA")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("countdown-timer").join("data.json")
}
