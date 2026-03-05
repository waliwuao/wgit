use crate::{git, utils};
use anyhow::{Result, bail};
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    println!("Solve workflow: check for remaining conflicts, stage all, then commit with a structured message.");

    let with_markers = git::files_with_conflict_markers(cwd)?;
    if !with_markers.is_empty() {
        println!("The following files still contain conflict markers (<<<<<<<):");
        for p in &with_markers {
            println!("  {p}");
        }
        bail!("Resolve all conflicts and remove markers, then run `wgit solve` again.");
    }

    let unmerged = git::unmerged_files(cwd)?;
    if !unmerged.is_empty() {
        println!("Staging {} previously unmerged file(s).", unmerged.len());
    }
    git::add_all(cwd)?;

    if !git::has_staged_changes(cwd)? {
        println!("No changes to commit after staging. Working tree may already be clean.");
        return Ok(());
    }

    let draft = utils::edit_commit_message("merge")?;
    let Some(draft) = draft else {
        println!("Commit editor canceled. Staged changes are unchanged.");
        return Ok(());
    };
    if draft.subject.trim().is_empty() {
        bail!("commit subject cannot be empty");
    }
    let msg = if draft.scope.trim().is_empty() {
        format!("merge: {}", draft.subject.trim())
    } else {
        format!("merge({}): {}", draft.scope.trim(), draft.subject.trim())
    };
    let full_msg = if draft.body.trim().is_empty() {
        msg.clone()
    } else {
        format!("{msg}\n\n{}", draft.body.trim())
    };
    git::commit_with_message(cwd, &full_msg)?;
    println!("Commit completed.");
    Ok(())
}
