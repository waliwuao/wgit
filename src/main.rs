mod cli;
mod config;
mod git;
mod utils;
mod commands;

use clap::Parser;
use cli::{Cli, SubCommand, BranchArgs};
use inquire::Select;
use utils::get_theme;
use colored::Colorize;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => interactive_select()?,
    };

    match command {
        SubCommand::Init => commands::init::run()?,
        SubCommand::Add => commands::add::run()?,
        SubCommand::Commit => commands::commit::run()?,
        SubCommand::Sync => commands::sync::run()?,
        SubCommand::Branch(args) => commands::branch::run(args)?,
        SubCommand::Undo => commands::undo::run()?,
        SubCommand::Config => commands::config::run()?,
    }

    Ok(())
}

fn interactive_select() -> anyhow::Result<SubCommand> {
    let options = vec![
        format!("{:<7} {}", "Init", "- Initialize git flow".bright_black()),
        format!("{:<7} {}", "Add", "- Interactive add files".bright_black()),
        format!("{:<7} {}", "Commit", "- Interactive commit".bright_black()),
        format!("{:<7} {}", "Sync", "- Smart pull & push".bright_black()),
        format!("{:<7} {}", "Branch", "- Manage branches (Switch/Delete/Start/Finish)".bright_black()),
        format!("{:<7} {}", "Undo", "- Undo recent changes".bright_black()),
        format!("{:<7} {}", "Config", "- Configure wgit settings".bright_black()),
    ];

    let choice = Select::new("Select a command to run (Esc to Exit):", options)
        .with_render_config(get_theme())
        .with_page_size(10) // 扩大展示数量，避免翻页
        .prompt()?;
    
    let cmd_str = choice.split_whitespace().next().unwrap_or("");

    match cmd_str {
        "Init" => Ok(SubCommand::Init),
        "Add" => Ok(SubCommand::Add),
        "Commit" => Ok(SubCommand::Commit),
        "Sync" => Ok(SubCommand::Sync),
        "Branch" => Ok(SubCommand::Branch(BranchArgs { action: None })), // 传入 None，让 Branch 命令自己处理交互
        "Undo" => Ok(SubCommand::Undo),
        "Config" => Ok(SubCommand::Config),
        _ => anyhow::bail!("Invalid command selected"),
    }
}