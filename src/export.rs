use crate::generator::GameRules;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn export_run_config(rules: &GameRules, seed: u64) -> Result<String, String> {
    let path = format!("assets/config/run_seed_{}.json", seed);
    let text = serde_json::to_string_pretty(rules)
        .map_err(|e| format!("Failed to serialize rules for export: {e}"))?;
    fs::write(&path, text)
        .map_err(|e| format!("Failed to write export file {path}: {e}"))?;
    println!("Exported run config to {path}");
    Ok(path)
}

pub fn save_preset(rules: &GameRules, name: &str) -> Result<String, String> {
    let dir = Path::new("assets/config/presets");
    if let Err(e) = fs::create_dir_all(dir) {
        return Err(format!("Failed to create presets dir {dir:?}: {e}"));
    }

    let path = dir.join(format!("{name}.json"));
    let text = serde_json::to_string_pretty(rules)
        .map_err(|e| format!("Failed to serialize rules for preset: {e}"))?;
    fs::write(&path, text)
        .map_err(|e| format!("Failed to write preset file {:?}: {e}", path))?;
    println!("Saved preset to {:?}", path);
    Ok(path.to_string_lossy().into_owned())
}

pub fn load_preset(name: &str) -> Result<GameRules, String> {
    let path = format!("assets/config/presets/{}.json", name);
    let text =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read preset file {path}: {e}"))?;
    let rules: GameRules = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse preset file {path}: {e}"))?;
    println!("Loaded preset from {path}");
    Ok(rules)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RunCartridge {
    pub seed: u64,
    pub level_reached: u32,
    pub score: u32,
    pub time_seconds: f32,
    pub rules: GameRules,
}

pub fn export_run_cartridge(
    rules: &GameRules,
    seed: u64,
    level_reached: u32,
    score: u32,
    time_seconds: f32,
) -> Result<String, String> {
    let dir = Path::new("assets/config/runs");
    if let Err(e) = fs::create_dir_all(dir) {
        return Err(format!("Failed to create runs dir {dir:?}: {e}"));
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let path = dir.join(format!("run_{}_{}.json", seed, timestamp));
    let cartridge = RunCartridge {
        seed,
        level_reached,
        score,
        time_seconds,
        rules: rules.clone(),
    };

    let text = serde_json::to_string_pretty(&cartridge)
        .map_err(|e| format!("Failed to serialize run cartridge: {e}"))?;
    fs::write(&path, text)
        .map_err(|e| format!("Failed to write run cartridge {:?}: {e}", path))?;
    println!("Exported run cartridge to {:?}", path);
    Ok(path.to_string_lossy().into_owned())
}

pub fn list_run_cartridges() -> Result<Vec<String>, String> {
    let dir = Path::new("assets/config/runs");
    let mut files: Vec<String> = Vec::new();

    match fs::read_dir(dir) {
        Ok(read_dir) => {
            for entry in read_dir {
                let entry = entry.map_err(|e| format!("Failed to read runs dir entry: {e}"))?;
                let path: PathBuf = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "json" {
                            if let Some(name) = path.file_name() {
                                files.push(name.to_string_lossy().into_owned());
                            }
                        }
                    }
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // No runs directory yet -> no cartridges.
        }
        Err(e) => {
            return Err(format!("Failed to read runs dir {dir:?}: {e}"));
        }
    }

    files.sort();
    Ok(files)
}

pub fn load_run_cartridge(name: &str) -> Result<RunCartridge, String> {
    let path = Path::new("assets/config/runs").join(name);
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read run cartridge {:?}: {e}", path))?;
    let cartridge: RunCartridge =
        serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse run cartridge {:?}: {e}", path))?;
    Ok(cartridge)
}
