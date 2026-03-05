use crate::{git, utils};
use anyhow::{Result, bail};
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    println!(
        "Finish workflow: detect parent branch, squash-merge current branch, commit, then clean up."
    );
    let source_branch = git::current_branch(cwd)?;

    if source_branch == "main" || source_branch == "master" {
        println!("Finish is not allowed on protected branch `{source_branch}`.");
        return Ok(());
    }

    if !git::is_clean_worktree(cwd)? {
        println!("Please commit or stash your changes before running finish.");
        return Ok(());
    }

    let parent = git::detect_parent_branch(cwd, &source_branch)?;
    println!("Current branch: {source_branch}");
    println!("Detected parent branch: {parent}");

    let confirmed = utils::confirm(&format!(
        "Squash-merge `{source_branch}` into `{parent}` and delete `{source_branch}`?"
    ))?;
    if !confirmed {
        println!("Finish canceled.");
        return Ok(());
    }

    git::checkout_branch(cwd, &parent)?;

    if let Err(error) = git::squash_merge_branch(cwd, &source_branch) {
        println!("Squash merge encountered conflicts.");
        let choice = utils::select_one(
            "Conflict option",
            &["abort".to_string(), "continue".to_string()],
        )?;
        match choice {
            Some(0) => {
                let _ = git::merge_abort(cwd);
                let should_reset = utils::confirm(
                    "[Safety Check] Also run `git reset --hard HEAD` to discard conflicted working tree changes?",
                )?;
                if should_reset {
                    let _ = git::reset_hard_head(cwd);
                }
                git::checkout_branch(cwd, &source_branch)?;
                println!("Merge aborted and branch restored.");
                return Ok(());
            }
            Some(1) => {
                println!("Resolve conflicts manually, then run:");
                println!("  wgit solve");
                println!("(solve will check for remaining markers, stage all, and prompt for commit message)");
                println!("Original git error: {error:#}");
                return Ok(());
            }
            _ => {
                println!("No conflict option selected. Keeping current merge state.");
                println!("Original git error: {error:#}");
                return Ok(());
            }
        }
    }

    let draft = utils::edit_commit_message("merge")?;
    let Some(draft) = draft else {
        println!("Merge commit editor canceled.");
        let should_reset = utils::confirm(
            "[Safety Check] Discard staged squash changes with `git reset --hard HEAD`?",
        )?;
        if should_reset {
            let _ = git::reset_hard_head(cwd);
        } else {
            println!("Keeping current working tree state. Clean it up manually if needed.");
        }
        let _ = git::checkout_branch(cwd, &source_branch);
        return Ok(());
    };
    if draft.subject.trim().is_empty() {
        bail!("merge subject cannot be empty");
    }
    let msg = if draft.scope.trim().is_empty() {
        format!("merge: {}", draft.subject.trim())
    } else {
        format!("merge({}): {}", draft.scope.trim(), draft.subject.trim())
    };
    let full_msg = if draft.body.trim().is_empty() {
        msg
    } else {
        format!("{msg}\n\n{}", draft.body.trim())
    };
    git::commit_with_message(cwd, &full_msg)?;

    if parent == "main" || parent == "master" {
        let last_tag = git::latest_tag(cwd)?;
        if let Some(tag) = &last_tag {
            println!("Latest tag: {tag}");
        } else {
            println!("No tag found yet.");
        }
        let new_tag = utils::input_text("New release tag (example: v1.2.3)")?;
        let new_tag = new_tag.trim();
        if new_tag.is_empty() {
            bail!("tag is required when finishing into protected main branch");
        }
        if !is_valid_tag(new_tag) {
            bail!("invalid tag format: {new_tag}. expected vMAJOR.MINOR.PATCH");
        }
        if git::tag_exists(cwd, new_tag)? {
            bail!("tag already exists: {new_tag}");
        }
        if let Some(prev) = last_tag {
            let prev_clean = prev.trim_start_matches('v');
            let new_clean = new_tag.trim_start_matches('v');
            if !is_newer(new_clean, prev_clean) {
                bail!("new tag must be higher than latest tag `{prev}`");
            }
        }
        git::create_tag(cwd, new_tag)?;
        println!("Tag `{new_tag}` created.");
    }

    let deleted_local = delete_source_branch(cwd, &source_branch)?;
    if deleted_local {
        maybe_delete_remote_branch(cwd, &source_branch)?;
    }
    println!("Finished `{source_branch}` into `{parent}`.");
    Ok(())
}

fn delete_source_branch(cwd: &Path, source_branch: &str) -> Result<bool> {
    if git::try_delete_branch(cwd, source_branch, false)? {
        return Ok(true);
    }

    println!(
        "Branch `{source_branch}` is not fully merged in Git history (common after squash merge)."
    );
    let force = utils::confirm(
        "[Safety Check] Force delete this local branch with `git branch -D`?",
    )?;
    if !force {
        println!("Branch cleanup skipped. You can remove it later with `wgit delete`.");
        return Ok(false);
    }

    let typed = utils::input_text(&format!(
        "[Safety Check] Type `{source_branch}` to confirm force delete"
    ))?;
    if typed.trim() != source_branch {
        println!("Branch name mismatch. Skip force delete.");
        return Ok(false);
    }

    git::delete_branch_force(cwd, source_branch)?;
    println!("Force deleted `{source_branch}`.");
    Ok(true)
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

fn is_valid_tag(tag: &str) -> bool {
    let t = tag.strip_prefix('v').unwrap_or(tag);
    let mut parts = t.split('.');
    let a = parts.next();
    let b = parts.next();
    let c = parts.next();
    if parts.next().is_some() {
        return false;
    }
    [a, b, c]
        .iter()
        .all(|part| part.is_some_and(|s| !s.is_empty() && s.chars().all(|ch| ch.is_ascii_digit())))
}

fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |value: &str| -> Vec<u64> {
        value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect()
    };
    let mut left = parse(candidate);
    let mut right = parse(current);
    let max_len = left.len().max(right.len());
    left.resize(max_len, 0);
    right.resize(max_len, 0);
    left > right
}
