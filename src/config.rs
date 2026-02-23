use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use crate::git;

#[derive(Serialize, Deserialize, Clone)]
pub struct WgitConfig {
    pub remotes: HashMap<String, String>,
    pub review_mode: ReviewMode,
    pub main_branch: String,
    pub dev_branch: String,
}

impl Default for WgitConfig {
    fn default() -> Self {
        Self {
            remotes: HashMap::new(),
            review_mode: ReviewMode::LocalMerge,
            main_branch: "main".to_string(),
            dev_branch: "develop".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Clone)]
pub enum ReviewMode {
    #[default]
    LocalMerge,
    RemoteReview,
}

pub fn get_config_path() -> anyhow::Result<std::path::PathBuf> {
    let root_result = git::get_output(&["rev-parse", "--show-toplevel"]);
    let root = match root_result {
        Ok(r) => r,
        Err(_) => std::env::current_dir()?.to_string_lossy().to_string(),
    };
    Ok(std::path::PathBuf::from(root).join(".wgit.json"))
}

pub fn load_config() -> anyhow::Result<WgitConfig> {
    let path = get_config_path()?;
    if path.exists() {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        Ok(WgitConfig::default())
    }
}

pub fn save_config(config: &WgitConfig) -> anyhow::Result<()> {
    let path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}