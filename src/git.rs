use std::process::{Command, Stdio};
use colored::*;
use anyhow::{Result, Context};

pub fn run_git(args: &[&str]) -> Result<()> {
    println!("{}", format!("> git {}", args.join(" ")).bright_cyan());
    
    let status = Command::new("git")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute git command")?;

    if !status.success() {
        anyhow::bail!("Git command failed with status: {}", status);
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