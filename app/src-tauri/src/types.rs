//! Serde types shared with the React frontend. All use camelCase to match the
//! JS command contract in `app/src/lib/engine.js`.

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub mode: String, // "setup" | "config"
    pub installed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub detail: String,
    /// A failing fatal check blocks proceeding.
    pub fatal: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preflight {
    pub can_proceed: bool,
    pub checks: Vec<Check>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    pub device: String,
    pub model: String,
    pub total_gib: f64,
    pub steamos_used_gib: f64,
    pub free_gib: f64,
    pub min_windows_gib: f64,
    pub max_windows_gib: f64,
    pub shrinkable_gib: f64,
    /// Current size of the SteamOS home partition (what we shrink). Internal use.
    #[serde(skip)]
    pub home_gib: f64,
    #[serde(skip)]
    pub home_part: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub n: u32,
    pub name: String,
    pub fs: String,
    pub gib: f64,
    pub role: String,
    #[serde(default)]
    pub added: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub before: Vec<Part>,
    pub after: Vec<Part>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsoInfo {
    pub ok: bool,
    pub edition: String,
    pub arch: String,
    pub build: String,
    pub detail: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub default_os: String, // "steamos" | "windows"
    pub timeout_seconds: u32,
    pub windows_gib: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config { default_os: "steamos".into(), timeout_seconds: 5, windows_gib: 0.0 }
    }
}

/// Progress payload emitted on the `install://progress` event.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub step: String,
    pub label: String,
    pub step_index: usize,
    pub step_count: usize,
    pub pct: u32,
    pub status: String, // "running" | "done" | "error"
    pub log: String,
}
