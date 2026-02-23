use crate::git;
use crate::config::load_config;
use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    let config = load_config()?;
    
    println!("\n{}", "── Initializing Git Flow ───────────────────────────────────────────────".cyan());

    println!("{} Creating repository...", " 1/4 ".on_cyan().black());
    git::run_git(&["init", &format!("--initial-branch={}", config.main_branch)])?;

    if git::get_output(&["rev-parse", "HEAD"]).is_err() {
        println!("{} Creating initial commit...", " 2/4 ".on_cyan().black());
        git::run_git(&["commit", "--allow-empty", "-m", "chore: initial wgit commit"])?;
    }

    println!("{} Setting up branch structure...", " 3/4 ".on_cyan().black());
    git::run_git(&["branch", "-M", &config.main_branch])?;
    let branches = git::get_output(&["branch", "--format=%(refname:short)"])?;
    if !branches.lines().any(|b| b.trim() == config.dev_branch) {
        git::run_git(&["branch", &config.dev_branch])?;
    }
    git::run_git(&["checkout", &config.dev_branch])?;

    println!("{} Installing protection hooks...", " 4/4 ".on_cyan().black());
    install_hook(&config.main_branch, &config.dev_branch)?;

    println!("{}\n", "wgit flow initialized successfully!".green().bold());
    Ok(())
}

pub fn install_hook(main: &str, dev: &str) -> anyhow::Result<()> {
    let hook_path = std::path::PathBuf::from(".git/hooks/pre-commit");
    let hook_script = format!(r#"#!/bin/sh
branch="$(git rev-parse --abbrev-ref HEAD)"
if [ "$branch" = "{}" ] || [ "$branch" = "{}" ]; then
    echo "wgit Error: Direct commits to $branch are forbidden."
    exit 1
fi
"#, main, dev);

    std::fs::write(&hook_path, hook_script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }
    Ok(())
}