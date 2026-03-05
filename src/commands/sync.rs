use crate::{git, utils};
use anyhow::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    println!(
        "Sync workflow: auto-stash local changes if needed, pull --rebase, push, then restore stash."
    );
    let branch = git::current_branch(cwd)?;
    let mut stashed = false;

    if git::has_uncommitted_changes(cwd)? {
        println!("Detected uncommitted changes. Creating temporary stash...");
        stashed = git::stash_push(cwd, "wgit sync auto-stash")?;
        if stashed {
            println!("Working tree stashed.");
        } else {
            println!("No changes needed stashing.");
        }
    }

    let upstream = git::upstream_remote_and_branch(cwd)?;
    let mut push_remote: Option<String> = None;

    let pull_result = match &upstream {
        Some((remote, upstream_branch)) => {
            println!("Using upstream branch: {remote}/{upstream_branch}");
            if !git::remote_branch_exists(cwd, remote, upstream_branch)? {
                println!("Remote branch does not exist yet (e.g. first push). Skipping pull.");
                Ok(())
            } else {
                git::pull_rebase(cwd, None, None)
            }
        }
        None => {
            let remotes = git::list_remotes(cwd)?;
            if remotes.is_empty() {
                println!("No remote configured. Skipping pull and push.");
                restore_stash_if_needed(cwd, stashed)?;
                return Ok(());
            }
            let labels: Vec<String> = remotes
                .iter()
                .map(|r| format!("{} -> {}", r.name, r.url))
                .collect();
            let selected = utils::select_one("Select remote for first sync", &labels)?;
            let Some(index) = selected else {
                println!("Sync canceled.");
                restore_stash_if_needed(cwd, stashed)?;
                return Ok(());
            };
            let remote_name = remotes[index].name.clone();
            push_remote = Some(remote_name.clone());
            if !git::remote_branch_exists(cwd, &remote_name, &branch)? {
                println!("Remote branch does not exist yet (e.g. first push). Skipping pull.");
                Ok(())
            } else {
                git::pull_rebase(cwd, Some(&remote_name), Some(&branch))
            }
        }
    };

    if let Err(error) = pull_result {
        println!("Pull with rebase failed.");
        let choice = utils::select_one(
            "Resolve option",
            &["abort".to_string(), "continue".to_string()],
        )?;
        match choice {
            Some(0) => {
                let _ = git::rebase_abort(cwd);
                restore_stash_if_needed(cwd, stashed)?;
                println!("Sync aborted and rebase state cleaned.");
                return Ok(());
            }
            Some(1) => {
                println!("Resolve conflicts manually, then run:");
                println!("  git add <files>");
                println!("  git rebase --continue");
                println!("Then run `wgit sync` again.");
                println!("Original git error: {error:#}");
                return Ok(());
            }
            _ => {
                println!("No option selected. Keeping current rebase state.");
                println!("Original git error: {error:#}");
                return Ok(());
            }
        }
    }

    git::push_current(cwd, push_remote.as_deref(), &branch)?;
    println!("Pull and push completed.");
    restore_stash_if_needed(cwd, stashed)?;
    println!("Sync completed.");
    Ok(())
}

fn restore_stash_if_needed(cwd: &Path, stashed: bool) -> Result<()> {
    if !stashed {
        return Ok(());
    }
    println!("Restoring stashed changes...");
    if let Err(error) = git::stash_pop(cwd) {
        println!("Failed to apply stash automatically. Resolve conflicts manually.");
        return Err(error);
    }
    println!("Stash restored.");
    Ok(())
}
