use std::process::{Command, Stdio};
use colored::*;
use anyhow::{Result, Context};

pub fn run_git(args: &[&str]) -> Result<()> {
    let cmd_display = format!("  git {}", args.join(" ")).bright_black();
    println!("{}", cmd_display);
    
    let output = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to execute git command")?
        .wait_with_output()?;

    if !output.status.success() {
        println!("{}", "── Git Error ───────────────────────────────────────────────────────────".red());
        let err = String::from_utf8_lossy(&output.stderr);
        println!("{}", err.trim());
        println!("{}", "────────────────────────────────────────────────────────────────────────".red());
        anyhow::bail!("Git execution failed.");
    }
    Ok(())
}

pub fn get_output(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", err);
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn is_dirty() -> bool {
    let out = get_output(&["status", "--porcelain"]).unwrap_or_default();
    !out.trim().is_empty()
}

pub fn ensure_clean() -> Result<()> {
    if is_dirty() {
        anyhow::bail!("Workspace has uncommitted changes. Please commit or stash them first.");
    }
    Ok(())
}