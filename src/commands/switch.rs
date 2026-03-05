use crate::{git, utils};
use anyhow::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    let current = git::current_branch(cwd)?;
    let mut branches = git::list_local_branches(cwd)?;
    if branches.is_empty() {
        println!("No local branches available.");
        return Ok(());
    }

    branches.sort();
    let labels: Vec<String> = branches
        .iter()
        .map(|branch| {
            if branch == &current {
                format!("{branch} (current)")
            } else {
                branch.clone()
            }
        })
        .collect();

    let selected = utils::select_one("Select branch to switch", &labels)?;
    let Some(selected_idx) = selected else {
        println!("Switch canceled.");
        return Ok(());
    };

    let target = &branches[selected_idx];
    if target == &current {
        println!("Already on `{current}`.");
        return Ok(());
    }

    if !git::is_clean_worktree(cwd)? {
        let confirmed = utils::confirm(
            "Working tree is not clean. Continue switching branch anyway (possible conflicts)?",
        )?;
        if !confirmed {
            println!("Switch canceled due to uncommitted changes.");
            return Ok(());
        }
    }

    git::checkout_branch(cwd, target)?;
    println!("Switched from `{current}` to `{target}`.");
    Ok(())
}
