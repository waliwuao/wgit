use crate::git;
use crate::config::load_config;
use inquire::Select;

pub fn run() -> anyhow::Result<()> {
    let config = load_config()?;
    if config.remotes.is_empty() {
        anyhow::bail!("No remotes configured in wgit. Please run `wgit config` to add a remote url.");
    }

    let remote_name = if config.remotes.len() == 1 {
        config.remotes.keys().next().unwrap().clone()
    } else {
        let remotes: Vec<_> = config.remotes.keys().cloned().collect();
        Select::new("Select remote to sync:", remotes).prompt()?
    };

    let current_branch = git::get_output(&["rev-parse", "--abbrev-ref", "HEAD"])?;

    println!("Intelligently syncing branch: {} with remote: {}", current_branch, remote_name);
    
    let _ = git::run_git(&["pull", &remote_name, &current_branch]);
    git::run_git(&["push", &remote_name, &current_branch])?;

    Ok(())
}