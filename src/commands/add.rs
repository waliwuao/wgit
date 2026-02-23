use crate::git;
use crate::utils::get_theme;
use inquire::MultiSelect;

pub fn run() -> anyhow::Result<()> {
    let status = git::get_output(&["status", "--porcelain"])?;
    if status.is_empty() {
        println!("No modified or untracked files.");
        return Ok(());
    }

    let mut files = Vec::new();
    for line in status.lines() {
        if line.len() > 3 {
            files.push(line[3..].trim().to_string());
        }
    }

    let selected_files = MultiSelect::new(
        "Select files to add (Space: select, Enter: confirm, 'a': toggle all):", 
        files
    )
    .with_render_config(get_theme())
    .prompt()?;

    if selected_files.is_empty() {
        println!("No files selected.");
        return Ok(());
    }

    let mut args = vec!["add"];
    for f in &selected_files {
        args.push(f);
    }
    git::run_git(&args)?;

    Ok(())
}