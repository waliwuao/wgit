use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct WgitConfig {
    pub protected_branches: Vec<String>,
}

impl Default for WgitConfig {
    fn default() -> Self {
        Self {
            protected_branches: vec!["main".to_string()],
        }
    }
}

pub fn config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".git").join("wgit.toml")
}

pub fn ensure_default_config(repo_root: &Path) -> Result<WgitConfig> {
    let cfg = WgitConfig::default();
    let path = config_path(repo_root);

    if path.exists() {
        return Ok(cfg);
    }

    let content = "[safety]\nprotected_branches = [\"main\"]\n";
    fs::write(&path, content)
        .with_context(|| format!("failed to write config file: {}", path.display()))?;

    Ok(cfg)
}
