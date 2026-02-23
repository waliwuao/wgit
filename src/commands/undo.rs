use crate::git;
use crate::utils::get_theme;
use inquire::Select;

pub fn run() -> anyhow::Result<()> {
    let options = vec![
        "Undo by Commit (git log)", 
        "Undo by Operation (git reflog)"
    ];
    let choice = Select::new("Select undo method:", options)
        .with_render_config(get_theme())
        .prompt()?;

    let lines = if choice.contains("log") {
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
        "--soft (Keep changes in staging)", 
        "--mixed (Keep changes in working directory)", 
        "--hard (Discard all changes completely)"
    ];
    let mode_choice = Select::new("Select reset mode:", modes)
        .with_render_config(get_theme())
        .prompt()?;
    let mode = mode_choice.split_whitespace().next().unwrap();

    git::run_git(&["reset", mode, hash])?;
    Ok(())
}