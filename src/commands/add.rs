use crate::{git, utils};
use anyhow::Result;
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    let files = git::stageable_files(cwd)?;
    if files.is_empty() {
        println!("No changes to stage.");
        return Ok(());
    }

    let selected = utils::select_many("Select files to stage", &files)?;
    if selected.is_empty() {
        println!("Staging canceled.");
        return Ok(());
    }

    let picked_files: Vec<String> = selected.into_iter().map(|idx| files[idx].clone()).collect();
    let mut args: Vec<&str> = vec!["add", "--"];
    for file in &picked_files {
        args.push(file.as_str());
    }
    git::run_git_in_dir(&args, cwd)?;

    println!("Staged {} file(s).", picked_files.len());
    Ok(())
}
