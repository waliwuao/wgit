use std::process::{Command, Stdio};
use colored::*;
use anyhow::{Result, Context};
use inquire::Select;

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
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git execution failed: {}", err.trim());
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

pub fn get_conflicted_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["diff", "--name-only", "--diff-filter=U"])
        .output()?;
    
    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = out_str
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(files)
    } else {
        Ok(vec![])
    }
}

pub fn resolve_interactive(operation: &str) -> Result<()> {
    println!("\n{}", format!("── Conflict Detected during {} ────────────────────────────────", operation).red().bold());

    loop {
        let files = get_conflicted_files()?;
        
        if files.is_empty() {
            println!("{}", "No conflicting files detected.".green());
            break;
        }

        println!("\n{}", "The following files have merge conflicts:".yellow());
        for (i, file) in files.iter().enumerate() {
            println!("  {}. {}", i + 1, file.red());
        }

        println!("\n{}", "GUIDE:".cyan().bold());
        println!("  1. Open the files above and resolve conflicts (look for <<<<<<< HEAD).");
        println!("  2. Mark them as resolved by running: {}", "git add <file>".green());
        println!("  3. Return here and select 'Continue'.\n");

        let choices = vec![
            "Continue (I have resolved and staged changes)",
            "Abort Operation (Exit wgit)",
        ];

        let selection = Select::new("Action:", choices)
            .with_render_config(crate::utils::get_theme())
            .prompt()?;

        if selection.starts_with("Abort") {
            anyhow::bail!("Operation aborted by user due to conflicts.");
        }

        let remaining_conflicts = get_conflicted_files()?;
        if !remaining_conflicts.is_empty() {
            println!("\n{}", "(!) Conflicts still exist in the following files:".on_red().white().bold());
            for file in remaining_conflicts {
                println!("  - {}", file);
            }
            println!("{}", "Please fix and 'git add' them before continuing.".yellow());
        } else {
            println!("{}", "Conflicts resolved successfully!".green());
            break;
        }
    }

    Ok(())
}