use crate::{git, utils};
use anyhow::{Result, bail};
use std::path::Path;

fn is_valid_branch_name(input: &str) -> bool {
    if input.is_empty() || input.contains(' ') || input.ends_with('/') || input.starts_with('/') {
        return false;
    }
    !input.contains("..") && !input.contains("~") && !input.contains("^") && !input.contains(':')
}

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    if !git::has_commits(cwd)? {
        println!("No commit history detected. Creating bootstrap empty commit...");
        git::create_empty_commit(cwd, "chore: bootstrap repository baseline")?;
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
        if !is_valid_branch_name(name) {
            bail!("invalid branch name: {name}");
        }
        let full_branch = format!("{}/{}", branch_types[index], name);
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
