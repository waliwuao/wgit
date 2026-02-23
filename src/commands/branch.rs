use crate::git;
use crate::cli::{BranchArgs, BranchAction};
use crate::utils::get_theme;
use inquire::{Select, Text};

pub fn run(args: BranchArgs) -> anyhow::Result<()> {
    match args.action {
        BranchAction::Switch => switch()?,
        BranchAction::Delete => delete()?,
        BranchAction::Start => start()?,
        BranchAction::Finish => finish()?,
    }
    Ok(())
}

fn get_branches() -> anyhow::Result<Vec<String>> {
    let out = git::get_output(&["branch", "--format=%(refname:short)"])?;
    Ok(out.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

fn switch() -> anyhow::Result<()> {
    let branches = get_branches()?;
    let branch = Select::new("Select branch to switch to:", branches)
        .with_render_config(get_theme())
        .prompt()?;
    git::run_git(&["checkout", &branch])?;
    Ok(())
}

fn delete() -> anyhow::Result<()> {
    let branches = get_branches()?;
    let filter_branches: Vec<_> = branches.into_iter().filter(|b| b != "main" && b != "develop").collect();
    
    if filter_branches.is_empty() {
        anyhow::bail!("No deletable branches available.");
    }

    let branch = Select::new("Select branch to delete:", filter_branches)
        .with_render_config(get_theme())
        .prompt()?;
    git::run_git(&["branch", "-D", &branch])?;
    Ok(())
}

fn start() -> anyhow::Result<()> {
    let types = vec!["feature", "bugfix", "release", "hotfix"];
    let branch_type = Select::new("Select branch type:", types)
        .with_render_config(get_theme())
        .prompt()?;
    
    let name = Text::new("Enter branch name:")
        .with_render_config(get_theme())
        .prompt()?;
    
    let full_name = format!("{}/{}", branch_type, name);
    git::run_git(&["checkout", "-b", &full_name])?;
    Ok(())
}

fn finish() -> anyhow::Result<()> {
    let current = git::get_output(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    if current == "main" || current == "develop" {
        anyhow::bail!("Cannot finish main or develop branches.");
    }

    if current.starts_with("release/") || current.starts_with("hotfix/") {
        git::run_git(&["checkout", "main"])?;
        git::run_git(&["merge", "--no-ff", &current])?;

        let tag = Text::new("Enter release tag (e.g., v1.0.0):")
            .with_render_config(get_theme())
            .prompt()?;
        git::run_git(&["tag", &tag])?;

        git::run_git(&["checkout", "develop"])?;
        git::run_git(&["merge", "--no-ff", &current])?;
    } else {
        git::run_git(&["checkout", "develop"])?;
        git::run_git(&["merge", "--no-ff", &current])?;
    }

    println!("Branch {} finished successfully.", current);
    Ok(())
}