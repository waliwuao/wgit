use crate::git;
use crate::cli::{BranchArgs, BranchAction};
use crate::utils::{get_theme, run_commit_form};
use crate::config::load_config;
use inquire::{Select, Text};
use colored::Colorize;

pub fn run(args: BranchArgs) -> anyhow::Result<()> {
    let action = match args.action {
        Some(a) => a,
        None => interactive_select()?,
    };

    match action {
        BranchAction::Switch => switch()?,
        BranchAction::Delete => delete()?,
        BranchAction::Start => start()?,
        BranchAction::Finish => finish()?,
    }
    Ok(())
}

fn interactive_select() -> anyhow::Result<BranchAction> {
    let actions = vec![
        format!("{:<14} {}", "Switch", "Checkout an existing branch from the local list".bright_black()),
        format!("{:<14} {}", "Delete", "Remove a redundant branch (protected branches excluded)".bright_black()),
        format!("{:<14} {}", "Start", "Create a standardized feature or hotfix branch".bright_black()),
        format!("{:<14} {}", "Finish", "Merge current branch into target and clean up".bright_black()),
    ];

    let choice = Select::new("Select branch action:", actions)
        .with_render_config(get_theme())
        .with_page_size(10)
        .prompt()?;
    
    let action_str = choice.split_whitespace().next().unwrap_or("");
    
    match action_str {
        "Switch" => Ok(BranchAction::Switch),
        "Delete" => Ok(BranchAction::Delete),
        "Start" => Ok(BranchAction::Start),
        "Finish" => Ok(BranchAction::Finish),
        _ => anyhow::bail!("Invalid branch action selected"),
    }
}

fn get_branches() -> anyhow::Result<Vec<String>> {
    let out = git::get_output(&["branch", "--format=%(refname:short)"])?;
    Ok(out.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
}

fn switch() -> anyhow::Result<()> {
    let branches = get_branches()?;
    let branch = Select::new("Select target branch:", branches)
        .with_render_config(get_theme())
        .with_page_size(10)
        .prompt()?;
    git::run_git(&["checkout", &branch])?;
    Ok(())
}

fn delete() -> anyhow::Result<()> {
    let config = load_config()?;
    let branches = get_branches()?;
    let filter_branches: Vec<_> = branches.into_iter()
        .filter(|b| b != &config.main_branch && b != &config.dev_branch)
        .collect();
    
    if filter_branches.is_empty() {
        anyhow::bail!("No deletable branches found.");
    }

    let branch = Select::new("Select branch to remove:", filter_branches)
        .with_render_config(get_theme())
        .with_page_size(10)
        .prompt()?;
    git::run_git(&["branch", "-D", &branch])?;
    Ok(())
}

fn start() -> anyhow::Result<()> {
    let types = vec!["feature", "bugfix", "release", "hotfix"];
    let branch_type = Select::new("Select branch category:", types)
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
    let config = load_config()?;
    let current = git::get_output(&["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string();
    
    if current == config.main_branch || current == config.dev_branch {
        anyhow::bail!("Operation denied: Cannot finish a protected branch.");
    }

    let types = vec!["merge", "feat", "fix", "chore", "release", "docs", "style", "refactor", "perf", "test"];
    let commit_type = Select::new("Select merge type:", types)
        .with_render_config(get_theme())
        .prompt()?;

    let (scope, subject, body) = run_commit_form()?;

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

    println!("\n{}", "── Finishing Branch ────────────────────────────────────────────────────".cyan());

    if current.starts_with("release/") || current.starts_with("hotfix/") {
        println!("{} Merging into {}...", " 1/3 ".on_cyan().black(), config.main_branch);
        git::run_git(&["checkout", &config.main_branch])?;
        git::run_git(&["merge", "--no-ff", &current, "-m", &msg])?;

        let tag = Text::new("Assign version tag (e.g. v1.0.0):")
            .with_render_config(get_theme())
            .prompt()?;
        
        if !tag.trim().is_empty() {
            git::run_git(&["tag", "-a", tag.trim(), "-m", &msg])?;
        }

        println!("{} Merging into {}...", " 2/3 ".on_cyan().black(), config.dev_branch);
        git::run_git(&["checkout", &config.dev_branch])?;
        git::run_git(&["merge", "--no-ff", &current, "-m", &msg])?;
    } else {
        println!("{} Merging into {}...", " 1/2 ".on_cyan().black(), config.dev_branch);
        git::run_git(&["checkout", &config.dev_branch])?;
        git::run_git(&["merge", "--no-ff", &current, "-m", &msg])?;
    }

    let cleanup_step = if current.starts_with("release/") || current.starts_with("hotfix/") { " 3/3 " } else { " 2/2 " };
    println!("{} Cleaning up branch...", cleanup_step.on_cyan().black());
    git::run_git(&["branch", "-d", &current])?;

    println!("{}\n", format!("Successfully finished branch: {}", current).green().bold());
    Ok(())
}