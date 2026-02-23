use crate::git;
use crate::utils::get_theme;
use inquire::MultiSelect;

pub fn run() -> anyhow::Result<()> {
    let output = git::get_output(&["status", "--porcelain", "-z"])?;
    if output.is_empty() {
        println!("No modified or untracked files.");
        return Ok(());
    }

    let mut files = Vec::new();
    let entries: Vec<&str> = output.split('\0').collect();
    
    let mut i = 0;
    while i < entries.len() {
        let entry = entries[i];
        if entry.is_empty() {
            i += 1;
            continue;
        }

        let status_code = &entry[..2];
        let path = entry[3..].to_string();
        
        files.push(path);

        if status_code.contains('R') {
            i += 1; 
        }
        i += 1;
    }

    if files.is_empty() {
        println!("No files to add.");
        return Ok(());
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