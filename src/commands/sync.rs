use crate::git;
use crate::config::load_config;
use crate::utils::get_theme;
use inquire::Select;
use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    let config = load_config()?;
    if config.remotes.is_empty() {
        anyhow::bail!("No remotes configured. Please run `wgit config` to add a remote.");
    }

    let remote_name = if config.remotes.len() == 1 {
        config.remotes.keys().next().unwrap().clone()
    } else {
        let remotes: Vec<_> = config.remotes.keys().cloned().collect();
        Select::new("Select remote to sync:", remotes)
            .with_render_config(get_theme())
            .prompt()?
    };

    let current_branch = git::get_output(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    
    let was_dirty = git::is_dirty();

    println!("\n{}", "── Syncing Repository ──────────────────────────────────────────────────".cyan());

    if was_dirty {
        println!("{} Workspace dirty, stashing local changes...", " STASH ".on_cyan().black());
        git::run_git(&["stash", "push", "-u", "-m", "wgit sync: auto-stash"])?;
    }

    println!("{} Checking remote state...", " PREP  ".on_cyan().black());
    let remote_main_check = git::get_output(&["ls-remote", "--heads", &remote_name, &config.main_branch])?;
    if !remote_main_check.contains(&format!("refs/heads/{}", config.main_branch)) {
        git::run_git(&["push", "-u", &remote_name, &config.main_branch])?;
    }

    println!("{} Pulling from {}/{}...", " PULL  ".on_cyan().black(), remote_name, current_branch);
    git::run_git(&["pull", &remote_name, &current_branch])?;

    println!("{} Pushing to {}/{}...", " PUSH  ".on_cyan().black(), remote_name, current_branch);
    git::run_git(&["push", "-u", &remote_name, &current_branch])?;

    if was_dirty {
        println!("{} Restoring local changes...", " POP   ".on_cyan().black());
        if let Err(_) = git::run_git(&["stash", "pop"]) {
            println!("\n{}", "(!) Note: Auto-stash pop resulted in conflicts. Your changes are safe in 'git stash'.".yellow());
        }
    }

    println!("{}\n", "Done! Repository is up to date.".green().bold());
    Ok(())
}