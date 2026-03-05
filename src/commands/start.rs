use crate::{config, git, utils};
use anyhow::{Result, bail};
use std::path::Path;

fn is_basic_branch_name_valid(input: &str) -> bool {
    if input.is_empty() || input.contains(' ') || input.ends_with('/') || input.starts_with('/') {
        return false;
    }
    !input.contains("..") && !input.contains("~") && !input.contains("^") && !input.contains(':')
}

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    println!("Start workflow: pick branch type, validate branch name, then create and switch.");
    if !git::has_commits(cwd)? {
        let cfg = config::load_config(cwd)?;
        if cfg.auto_bootstrap_commit_on_start {
            println!("No commit history detected. Auto bootstrap commit is enabled.");
            git::create_empty_commit(cwd, "chore: bootstrap repository baseline")?;
        } else {
            println!("[Safety Check] No commit history detected.");
            let confirmed = utils::confirm(
                "Create an empty bootstrap commit now? (recommended before branch creation)",
            )?;
            if !confirmed {
                println!("Start canceled. Create first commit, then run `wgit start` again.");
                return Ok(());
            }
            git::create_empty_commit(cwd, "chore: bootstrap repository baseline")?;
        }
    }

    let branch_types = vec![
        "feature".to_string(),
        "bugfix".to_string(),
        "hotfix".to_string(),
        "release".to_string(),
    ];

    let selected = utils::select_one("Select branch type", &branch_types)?;
    if let Some(index) = selected {
        let raw_name = utils::input_text("Branch name")?;
        let name = raw_name.trim();
        if name.is_empty() {
            println!("Branch creation canceled: empty name.");
            return Ok(());
        }
        if !is_basic_branch_name_valid(name) {
            bail!("invalid branch name: {name}");
        }
        let full_branch = format!("{}/{}", branch_types[index], name);
        if !git::is_valid_branch_ref(cwd, &full_branch)? {
            bail!(
                "invalid branch name by git rules: {full_branch}. example: feature/login-form"
            );
        }
        if git::branch_exists(cwd, &full_branch)? {
            bail!("branch already exists: {full_branch}");
        }

        git::run_git_in_dir(&["checkout", "-b", &full_branch], cwd)?;
        println!("Created and switched to `{full_branch}`.");
    } else {
        println!("Start command canceled.");
    }
    Ok(())
}
