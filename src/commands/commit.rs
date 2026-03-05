use crate::{config, git, utils};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    println!("Commit workflow: validate staged changes, draft message, then create commit.");
    let current_branch = git::current_branch(cwd)?;
    if config::is_protected_branch(cwd, &current_branch)? {
        println!(
            "Commit is not allowed on protected branch `{}`.",
            current_branch.trim()
        );
        return Ok(());
    }

    if !git::has_staged_changes(cwd)? {
        println!("No staged changes found. Run `wgit add` first.");
        return Ok(());
    }

    let commit_types = vec![
        "feat".to_string(),
        "fix".to_string(),
        "docs".to_string(),
        "refactor".to_string(),
        "test".to_string(),
        "chore".to_string(),
    ];

    let selected = utils::select_one("Select commit type", &commit_types)?;
    if let Some(index) = selected {
        let commit_type = &commit_types[index];
        if let Some(draft) = utils::edit_commit_message(commit_type)? {
            if draft.subject.trim().is_empty() {
                bail!("commit subject cannot be empty");
            }

            let header = if draft.scope.is_empty() {
                format!("{commit_type}: {}", draft.subject.trim())
            } else {
                format!(
                    "{commit_type}({}): {}",
                    draft.scope.trim(),
                    draft.subject.trim()
                )
            };
            let full_message = if draft.body.trim().is_empty() {
                header.clone()
            } else {
                format!("{header}\n\n{}", draft.body.trim())
            };

            let mut tmp_path = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("failed to generate temp filename")?
                .as_nanos();
            tmp_path.push(format!("wgit-commit-{nanos}.txt"));
            fs::write(&tmp_path, full_message).with_context(|| {
                format!(
                    "failed to write commit message file: {}",
                    tmp_path.display()
                )
            })?;
            let tmp_arg = tmp_path.to_string_lossy().to_string();
            let commit_result = git::run_git_in_dir(&["commit", "-F", &tmp_arg], cwd);
            let _ = fs::remove_file(&tmp_path);
            commit_result?;

            if draft.scope.is_empty() {
                println!(
                    "Prepared commit message: {}: {}",
                    commit_type, draft.subject
                );
            } else {
                println!(
                    "Prepared commit message: {}({}): {}",
                    commit_type, draft.scope, draft.subject
                );
            }
            if !draft.body.is_empty() {
                println!("Body:\n{}", draft.body);
            }
            println!("Commit completed.");
        } else {
            println!("Commit editor canceled.");
        }
    } else {
        println!("Commit canceled.");
    }
    Ok(())
}
