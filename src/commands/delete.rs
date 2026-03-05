use crate::{config, git, utils};
use anyhow::{Result, bail};
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    println!("Delete workflow: choose local branch, try safe delete, then optional force delete.");

    let current = git::current_branch(cwd)?;
    let mut branches = git::list_local_branches(cwd)?;
    branches.retain(|branch| branch != &current);
    branches.sort();

    if branches.is_empty() {
        println!("No deletable local branches found.");
        return Ok(());
    }

    let labels: Vec<String> = branches
        .iter()
        .map(|branch| {
            let protected = config::is_protected_branch(cwd, branch).unwrap_or(false);
            if protected {
                format!("{branch} (protected)")
            } else {
                branch.clone()
            }
        })
        .collect();

    let selected = utils::select_one("Select branch to delete", &labels)?;
    let Some(index) = selected else {
        println!("Delete canceled.");
        return Ok(());
    };
    let target = &branches[index];

    if config::is_protected_branch(cwd, target)? {
        bail!("deleting protected branch is blocked: {target}");
    }

    let deleted_local = if git::try_delete_branch(cwd, target, false)? {
        println!("Deleted `{target}` with safe mode (`-d`).");
        true
    } else {
        println!("Safe delete failed. Branch may still contain unmerged commits.");
        let force = utils::confirm(
            "[Safety Check] Force delete with `git branch -D`? This can remove unmerged work.",
        )?;
        if !force {
            println!("Delete canceled.");
            return Ok(());
        }

        let typed = utils::input_text(&format!(
            "[Safety Check] Type `{target}` to confirm force delete"
        ))?;
        if typed.trim() != target {
            println!("Branch name mismatch. Delete canceled.");
            return Ok(());
        }

        git::delete_branch_force(cwd, target)?;
        println!("Force deleted `{target}`.");
        true
    };

    if deleted_local {
        maybe_delete_remote_branch(cwd, target)?;
    }
    Ok(())
}

fn maybe_delete_remote_branch(cwd: &Path, branch: &str) -> Result<()> {
    let remotes = git::list_remotes(cwd)?;
    if remotes.is_empty() {
        return Ok(());
    }

    let confirmed = utils::confirm(
        "Remote repositories detected. Delete remote branch too?",
    )?;
    if !confirmed {
        return Ok(());
    }

    let labels: Vec<String> = remotes
        .iter()
        .map(|entry| format!("{} -> {}", entry.name, entry.url))
        .collect();
    let selected = utils::select_one("Select remote to delete branch from", &labels)?;
    let Some(index) = selected else {
        println!("Remote delete canceled.");
        return Ok(());
    };
    let remote = &remotes[index].name;

    if !git::remote_branch_exists(cwd, remote, branch)? {
        println!("Remote branch `{remote}/{branch}` does not exist. Skip remote delete.");
        return Ok(());
    }

    git::delete_remote_branch(cwd, remote, branch)?;
    println!("Deleted remote branch `{remote}/{branch}`.");
    Ok(())
}
