use crate::git;
use crate::utils::get_theme;
use inquire::Select;
use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    let options = vec![
        format!("{:<14} {}", "Commit", "Undo changes by selecting a point from git log".bright_black()),
        format!("{:<14} {}", "Operation", "Undo changes by selecting an action from git reflog".bright_black())
    ];
    let choice = Select::new("Select undo method:", options)
        .with_render_config(get_theme())
        .prompt()?;

    let lines = if choice.contains("Commit") {
        git::get_output(&["log", "--oneline", "-n", "20"])?
    } else {
        git::get_output(&["reflog", "-n", "20"])?
    };

    let log_lines: Vec<&str> = lines.lines().collect();
    if log_lines.is_empty() {
        anyhow::bail!("No history found.");
    }

    let selected = Select::new("Select target point to reset to:", log_lines)
        .with_render_config(get_theme())
        .prompt()?;
    let hash = selected.split_whitespace().next().unwrap();

    let modes = vec![
        format!("{:<14} {}", "--soft", "Keep all changes in staging area".bright_black()),
        format!("{:<14} {}", "--mixed", "Keep all changes in working directory (Default)".bright_black()),
        format!("{:<14} {}", "--hard", "Discard all changes completely (Caution!)".bright_black())
    ];
    let mode_choice = Select::new("Select reset mode:", modes)
        .with_render_config(get_theme())
        .prompt()?;
    let mode = mode_choice.split_whitespace().next().unwrap();

    git::run_git(&["reset", mode, hash])?;
    Ok(())
}