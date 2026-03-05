use crate::{config, git, utils};
use anyhow::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    println!("Undo workflow: choose rollback base (commit/reflog) and reset level (soft/hard).");
    let base_types = vec!["by-commit".to_string(), "by-operation".to_string()];
    let level_types = vec!["soft".to_string(), "hard".to_string()];

    let base = utils::select_one("Select undo base", &base_types)?;
    let level = utils::select_one("Select undo level", &level_types)?;

    match (base, level) {
        (Some(base_idx), Some(level_idx)) => {
            let hard = level_idx == 1;
            if base_idx == 0 {
                undo_by_commit(cwd, hard)?;
            } else {
                undo_by_operation(cwd, hard)?;
            }
        }
        _ => println!("Undo canceled."),
    }
    Ok(())
}

fn undo_by_commit(cwd: &Path, hard: bool) -> Result<()> {
    let commits = git::list_recent_commits(cwd, 30)?;
    if commits.is_empty() {
        println!("No commit history found.");
        return Ok(());
    }

    let selected = utils::select_one("Select target commit", &commits)?;
    let Some(index) = selected else {
        println!("Undo canceled.");
        return Ok(());
    };
    let line = &commits[index];
    let Some((hash, _)) = line.split_once(' ') else {
        println!("Invalid commit entry: {line}");
        return Ok(());
    };

    if hard && !confirm_hard_reset(cwd, hash)? {
        println!("Undo canceled.");
        return Ok(());
    }
    git::reset_to(cwd, hash, hard)?;
    println!(
        "Undo complete: reset {} to commit {}.",
        if hard { "hard" } else { "soft" },
        hash
    );
    Ok(())
}

fn undo_by_operation(cwd: &Path, hard: bool) -> Result<()> {
    let reflog = git::list_reflog(cwd, 30)?;
    if reflog.is_empty() {
        println!("No reflog entries found.");
        return Ok(());
    }

    let labels: Vec<String> = reflog
        .iter()
        .map(|entry| format!("{} {}", entry.hash, entry.summary))
        .collect();
    let selected = utils::select_one("Select target operation", &labels)?;
    let Some(index) = selected else {
        println!("Undo canceled.");
        return Ok(());
    };

    let target = &reflog[index].hash;
    if hard && !confirm_hard_reset(cwd, target)? {
        println!("Undo canceled.");
        return Ok(());
    }
    git::reset_to(cwd, target, hard)?;
    println!(
        "Undo complete: reset {} to operation {}.",
        if hard { "hard" } else { "soft" },
        target
    );
    Ok(())
}

fn confirm_hard_reset(cwd: &Path, target: &str) -> Result<bool> {
    println!("[Safety Check] Hard reset will rewrite history and discard local working tree changes.");
    let confirmed = utils::confirm(&format!(
        "Continue hard reset to `{target}`?"
    ))?;
    if !confirmed {
        return Ok(false);
    }

    let cfg = config::load_config(cwd)?;
    if !cfg.require_double_confirm_for_hard_reset {
        return Ok(true);
    }

    let expected = target.chars().take(7).collect::<String>();
    let typed = utils::input_text(&format!(
        "[Safety Check] Type `{expected}` to confirm hard reset"
    ))?;
    Ok(typed.trim() == expected)
}
