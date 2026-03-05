use crate::{config, git};
use anyhow::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    let (has_main, has_master) = if git::is_git_repo(cwd)? {
        println!("Git repository detected.");
        let has_main = git::branch_exists(cwd, "main")?;
        let has_master = git::branch_exists(cwd, "master")?;

        if has_master && !has_main {
            println!("Detected default branch `master`. Renaming to `main`...");
            git::run_git_in_dir(&["branch", "-m", "master", "main"], cwd)?;
            println!("Branch renamed to `main`.");
            (true, false)
        } else if has_main {
            println!("Default branch `main` is already present.");
            (true, has_master)
        } else {
            println!("No `main` or `master` branch found yet.");
            println!("Binding HEAD to unborn `main` branch...");
            git::run_git_in_dir(&["symbolic-ref", "HEAD", "refs/heads/main"], cwd)?;
            (true, false)
        }
    } else {
        println!("No Git repository found. Initializing with main branch...");
        git::run_git(&["init", "-b", "main"])?;
        (true, false)
    };

    let current = git::current_branch(cwd)?;
    if current != "main" {
        if has_main {
            println!("Switching to `main` branch...");
            git::checkout_branch(cwd, "main")?;
        } else if has_master {
            println!("Creating and switching to `main` from current HEAD...");
            let (ok, _) = git::run_git_allow_fail_in_dir(&["checkout", "-b", "main"], cwd)?;
            if !ok {
                println!("Falling back to binding HEAD to unborn `main` branch...");
                git::run_git_in_dir(&["symbolic-ref", "HEAD", "refs/heads/main"], cwd)?;
            }
        }
    }

    let cfg = config::ensure_default_config(cwd)?;
    println!(
        "wgit initialized. Protected branches: {}",
        cfg.protected_branches.join(", ")
    );

    Ok(())
}
