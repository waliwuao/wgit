use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct WgitConfig {
    pub protected_branches: Vec<String>,
    pub auto_bootstrap_commit_on_start: bool,
    pub require_double_confirm_for_hard_reset: bool,
}

#[derive(Debug, Deserialize)]
struct RawWgitConfig {
    safety: Option<RawSafetyConfig>,
}

#[derive(Debug, Deserialize)]
struct RawSafetyConfig {
    protected_branches: Option<Vec<String>>,
    auto_bootstrap_commit_on_start: Option<bool>,
    require_double_confirm_for_hard_reset: Option<bool>,
}

impl Default for WgitConfig {
    fn default() -> Self {
        Self {
            protected_branches: vec!["main".to_string()],
            auto_bootstrap_commit_on_start: false,
            require_double_confirm_for_hard_reset: true,
        }
    }
}

pub fn config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(".git").join("wgit.toml")
}

pub fn load_config(repo_root: &Path) -> Result<WgitConfig> {
    let path = config_path(repo_root);
    if !path.exists() {
        return Ok(WgitConfig::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    let raw: RawWgitConfig = toml::from_str(&content)
        .with_context(|| format!("failed to parse config file: {}", path.display()))?;

    let safety = raw.safety;
    let mut protected_branches = safety
        .as_ref()
        .and_then(|value| value.protected_branches.clone())
        .unwrap_or_default()
        .into_iter()
        .map(|branch| branch.trim().to_string())
        .filter(|branch| !branch.is_empty())
        .collect::<Vec<_>>();

    if protected_branches.is_empty() {
        protected_branches = WgitConfig::default().protected_branches;
    }

    let auto_bootstrap_commit_on_start = safety
        .as_ref()
        .and_then(|value| value.auto_bootstrap_commit_on_start)
        .unwrap_or(WgitConfig::default().auto_bootstrap_commit_on_start);
    let require_double_confirm_for_hard_reset = safety
        .as_ref()
        .and_then(|value| value.require_double_confirm_for_hard_reset)
        .unwrap_or(WgitConfig::default().require_double_confirm_for_hard_reset);

    Ok(WgitConfig {
        protected_branches,
        auto_bootstrap_commit_on_start,
        require_double_confirm_for_hard_reset,
    })
}

pub fn ensure_default_config(repo_root: &Path) -> Result<WgitConfig> {
    let path = config_path(repo_root);

    if path.exists() {
        return load_config(repo_root);
    }

    let content = "[safety]\nprotected_branches = [\"main\"]\nauto_bootstrap_commit_on_start = false\nrequire_double_confirm_for_hard_reset = true\n";
    fs::write(&path, content)
        .with_context(|| format!("failed to write config file: {}", path.display()))?;

    load_config(repo_root)
}

pub fn is_protected_branch(repo_root: &Path, branch: &str) -> Result<bool> {
    let branch = branch.trim();
    let cfg = load_config(repo_root)?;
    Ok(cfg
        .protected_branches
        .iter()
        .any(|protected| protected.trim() == branch))
}
