// src/store.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Default)]
pub struct Store {
    pub frecency: HashMap<String, VisitStats>,
    pub pins: HashMap<String, String>,
    pub prev: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct VisitStats {
    pub visits: u64,
    pub last: u64,
}

impl Store {
    pub fn data_path() -> PathBuf {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".local/share"));
        base.join("jump").join("data.json")
    }

    pub fn load() -> Self {
        let path = Self::data_path();
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::data_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    pub fn record_visit(&mut self, path: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = self.frecency.entry(path.to_string()).or_default();
        entry.visits += 1;
        entry.last = now;
    }

    pub fn frecency_score(&self, path: &str) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        match self.frecency.get(path) {
            None => 0.0,
            Some(s) => {
                let age_days = (now.saturating_sub(s.last)) as f64 / 86400.0;
                let decay = if age_days < 1.0 {
                    4.0
                } else if age_days < 7.0 {
                    2.0
                } else if age_days < 30.0 {
                    0.5
                } else {
                    0.25
                };
                s.visits as f64 * decay
            }
        }
    }
}