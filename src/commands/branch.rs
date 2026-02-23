use crate::git;
use crate::cli::{BranchArgs, BranchAction};
use crate::utils::{get_theme, run_commit_form};
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
        format!("{:<7} {}", "Switch", "- Switch to another branch".bright_black()),
        format!("{:<7} {}", "Delete", "- Delete a branch".bright_black()),
        format!("{:<7} {}", "Start", "- Create a new feature/hotfix branch".bright_black()),
        format!("{:<7} {}", "Finish", "- Merge current branch into develop/main".bright_black()),
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
    let branch = Select::new("Select branch to switch to:", branches)
        .with_render_config(get_theme())
        .with_page_size(10)
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
        .with_page_size(10)
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

    // 引导用户输入提交信息，避免弹出 nano
    println!("Preparing merge commit message...");
    let types = vec!["merge", "feat", "fix", "chore", "release", "docs", "style", "refactor", "perf", "test"];
    let commit_type = Select::new("Select merge commit type:", types)
        .with_render_config(get_theme())
        .prompt()?;

    let (scope, subject, body) = run_commit_form()?;

    if subject.trim().is_empty() {
        anyhow::bail!("Subject cannot be empty");
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

    if current.starts_with("release/") || current.starts_with("hotfix/") {
        git::run_git(&["checkout", "main"])?;
        // 使用 -m 参数传入构建好的消息，防止编辑器弹出
        git::run_git(&["merge", "--no-ff", &current, "-m", &msg])?;

        let tag = Text::new("Enter release tag (e.g., v1.0.0):")
            .with_render_config(get_theme())
            .prompt()?;
        git::run_git(&["tag", &tag])?;

        git::run_git(&["checkout", "develop"])?;
        git::run_git(&["merge", "--no-ff", &current, "-m", &msg])?;
    } else {
        git::run_git(&["checkout", "develop"])?;
        git::run_git(&["merge", "--no-ff", &current, "-m", &msg])?;
    }

    println!("Branch {} finished successfully.", current);
    Ok(())
}