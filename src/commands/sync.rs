use crate::git;
use crate::config::load_config;
use crate::utils::get_theme;
use inquire::Select;
use colored::Colorize;
use std::path::Path;

struct StashGuard {
    active: bool,
}

impl StashGuard {
    fn new() -> Self {
        Self { active: false }
    }

    fn mark_active(&mut self) {
        self.active = true;
    }

    fn consume(mut self) {
        self.active = false;
    }
}

impl Drop for StashGuard {
    fn drop(&mut self) {
        if self.active {
            println!("\n{}", "(!) Process aborted while changes were stashed.".red().bold());
            println!("To recover your local changes, run: {}", "git stash pop".green().bold());
            println!("(Check 'git stash list' if you have multiple stashes)");
        }
    }
}

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
    let mut guard = StashGuard::new();

    println!("\n{}", "── Syncing Repository ──────────────────────────────────────────────────".cyan());

    if was_dirty {
        println!("{} Workspace dirty, stashing local changes...", " STASH ".on_cyan().black());
        git::run_git(&["stash", "push", "-u", "-m", "wgit sync: auto-stash"])?;
        guard.mark_active();
    }

    println!("{} Checking remote state...", " PREP  ".on_cyan().black());
    let remote_main_check = git::get_output(&["ls-remote", "--heads", &remote_name, &config.main_branch])?;
    if !remote_main_check.contains(&format!("refs/heads/{}", config.main_branch)) {
        git::run_git(&["push", "-u", &remote_name, &config.main_branch])?;
    }

    println!("{} Pulling from {}/{}...", " PULL  ".on_cyan().black(), remote_name, current_branch);
    if let Err(_) = git::run_git(&["pull", &remote_name, &current_branch]) {
        git::resolve_interactive("Sync Pull")?;
        
        let git_dir = git::get_output(&["rev-parse", "--git-dir"])?;
        let merge_head = Path::new(&git_dir).join("MERGE_HEAD");
        
        if merge_head.exists() {
            println!("{} Finalizing merge...", " COMMIT ".on_cyan().black());
            git::run_git(&["commit", "--no-edit"])?;
        }
    }

    println!("{} Pushing to {}/{}...", " PUSH  ".on_cyan().black(), remote_name, current_branch);
    git::run_git(&["push", "-u", &remote_name, &current_branch])?;

    if was_dirty {
        println!("{} Restoring local changes...", " POP   ".on_cyan().black());
        if let Err(_) = git::run_git(&["stash", "pop"]) {
            println!("\n{}", "(!) Conflict detected during stash apply.".yellow());
            match git::resolve_interactive("Stash Pop") {
                Ok(_) => {
                    println!("{}", "Stash changes restored and conflicts resolved.".green());
                    println!("{}", "Note: The stash entry is still kept. You can drop it with 'git stash drop' if verified.".bright_black());
                    guard.consume();
                },
                Err(e) => {
                    println!("\n{}", "(!) Stash pop aborted or failed.".red());
                    return Err(e);
                }
            }
        } else {
            guard.consume();
        }
    }

    println!("{}\n", "Done! Repository is up to date.".green().bold());
    Ok(())
}