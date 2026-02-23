use crate::git;
use crate::config::{load_config, ReviewMode};
use crate::utils::get_theme;
use inquire::Select;
use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    let types = vec!["feat", "fix", "docs", "style", "refactor", "perf", "test", "chore"];
    let commit_type = Select::new("Select commit category:", types)
        .with_render_config(get_theme())
        .prompt()?;

    let (scope, subject, body): (String, String, String) = crate::utils::run_commit_form()?;

    if subject.trim().is_empty() {
        anyhow::bail!("Required field 'Subject' is empty.");
    }

    let mut msg = format!("{commit_type}");
    if !scope.trim().is_empty() {
        msg.push_str(&format!("({})", scope.trim()));
    }
    msg.push_str(&format!(": {}", subject.trim()));

    if !body.trim().is_empty() {
        msg.push_str("\n\n");
        msg.push_str(body.trim());
    }

    git::run_git(&["commit", "-m", &msg])?;

    let config = load_config()?;
    if config.review_mode == ReviewMode::RemoteReview {
        println!("{}", "Review mode active: Preparing push for remote review...".bright_black());
        let current_branch = git::get_output(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        
        if config.remotes.is_empty() {
            println!("No remote targets configured. Push skipped.");
        } else {
            let remote = if config.remotes.len() == 1 {
                config.remotes.keys().next().unwrap().clone()
            } else {
                let remotes: Vec<_> = config.remotes.keys().cloned().collect();
                Select::new("Select remote target:", remotes)
                    .with_render_config(get_theme())
                    .prompt()?
            };
            git::run_git(&["push", "-u", &remote, &current_branch])?;
        }
    }

    Ok(())
}