use crate::git;
use anyhow::{Context, Result, anyhow, bail};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    let origin_url = git::origin_remote_url(cwd)?.ok_or_else(|| {
        anyhow!("remote.origin.url not found; set origin to enable `wgit update`")
    })?;
    let repo = git::github_repo_slug_from_url(&origin_url)
        .ok_or_else(|| anyhow!("origin remote is not a GitHub repository: {origin_url}"))?;

    let current = normalize_version(env!("CARGO_PKG_VERSION"));
    let release = fetch_latest_release(&repo)?;
    let latest = normalize_version(&release.tag_name);

    if !is_newer(&latest, &current) {
        println!("Already up to date (current: {current}, latest: {latest}).");
        return Ok(());
    }

    let asset_name = platform_asset_name();
    let Some(asset) = release.assets.iter().find(|a| a.name == asset_name) else {
        bail!(
            "release `{}` has no compatible asset `{asset_name}`",
            release.tag_name
        );
    };

    println!("New version available: {current} -> {latest}");
    println!("Downloading asset: {}", asset.name);
    let bytes = download_asset(&asset.browser_download_url)?;

    let exe_path = std::env::current_exe().context("failed to locate current executable")?;
    replace_executable(&exe_path, &bytes)?;
    println!("Update completed. Restart wgit to use version {latest}.");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

fn fetch_latest_release(repo: &str) -> Result<ReleaseResponse> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = Client::builder()
        .build()
        .context("failed to build HTTP client")?;

    let response = client
        .get(url)
        .header("User-Agent", "wgit-updater")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("failed to request latest release")?
        .error_for_status()
        .context("GitHub API returned non-success status")?;

    response
        .json::<ReleaseResponse>()
        .context("failed to parse release response")
}

fn download_asset(url: &str) -> Result<Vec<u8>> {
    let client = Client::builder()
        .build()
        .context("failed to build download client")?;
    let bytes = client
        .get(url)
        .header("User-Agent", "wgit-updater")
        .send()
        .with_context(|| format!("failed to download asset: {url}"))?
        .error_for_status()
        .context("asset download returned non-success status")?
        .bytes()
        .context("failed to read downloaded bytes")?;
    Ok(bytes.to_vec())
}

fn normalize_version(raw: &str) -> String {
    raw.trim()
        .trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
}

fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |value: &str| -> Vec<u64> {
        value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect()
    };

    let mut left = parse(candidate);
    let mut right = parse(current);
    let max_len = left.len().max(right.len());
    left.resize(max_len, 0);
    right.resize(max_len, 0);
    left > right
}

fn platform_asset_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "wgit-windows-amd64.exe"
    } else if cfg!(target_os = "macos") {
        "wgit-macos-amd64"
    } else {
        "wgit-linux-amd64"
    }
}

fn replace_executable(exe_path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = exe_path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = exe_path
        .file_name()
        .ok_or_else(|| anyhow!("invalid executable path: {}", exe_path.display()))?
        .to_string_lossy()
        .to_string();

    let new_path = temp_path(parent, &file_name, ".new");
    let backup_path = temp_path(parent, &file_name, ".bak");
    fs::write(&new_path, bytes)
        .with_context(|| format!("failed to write downloaded binary: {}", new_path.display()))?;
    set_executable_permission(&new_path)?;

    fs::rename(exe_path, &backup_path).with_context(|| {
        format!(
            "failed to backup current binary: {} -> {}",
            exe_path.display(),
            backup_path.display()
        )
    })?;

    if let Err(error) = fs::rename(&new_path, exe_path) {
        let _ = fs::rename(&backup_path, exe_path);
        let _ = fs::remove_file(&new_path);
        return Err(anyhow!(
            "failed to replace binary: {error}. original binary restored"
        ));
    }

    let _ = fs::remove_file(&backup_path);
    Ok(())
}

#[cfg(unix)]
fn set_executable_permission(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(path)
        .with_context(|| format!("failed to read file metadata: {}", path.display()))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to set executable bit: {}", path.display()))
}

#[cfg(not(unix))]
fn set_executable_permission(_path: &Path) -> Result<()> {
    Ok(())
}

fn temp_path(parent: &Path, file_name: &str, suffix: &str) -> PathBuf {
    parent.join(format!("{file_name}{suffix}"))
}
