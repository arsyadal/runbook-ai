use anyhow::Result;
use std::fs;

use crate::models::Config;
use crate::util::{project_root, STORAGE_DIR};

pub fn load_config() -> Result<Config> {
    let root = project_root()?;
    let path = root.join(STORAGE_DIR).join("config.json");
    if !path.exists() {
        return Ok(Config::default_for(&root));
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}
