mod cli;
mod config;
mod git;
mod utils;
mod commands;

use clap::Parser;
use cli::{Cli, SubCommand, BranchArgs, BranchAction};
use inquire::{Select, InquireError};
use utils::get_theme;
use colored::Colorize;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        execute_command(cmd)?;
    } else {
        loop {
            match interactive_select() {
                Ok(cmd) => {
                    if let Err(e) = execute_command(cmd) {
                        println!("Error: {}", e);
                    }
                }
                Err(e) => {
                    if let Some(InquireError::OperationCanceled) = e.downcast_ref::<InquireError>() {
                        println!("Bye!");
                        break;
                    }
                    return Err(e);
                }
            }
            println!(); 
        }
    }

    Ok(())
}

fn execute_command(command: SubCommand) -> anyhow::Result<()> {
    match command {
        SubCommand::Init => commands::init::run(),
        SubCommand::Add => commands::add::run(),
        SubCommand::Commit => commands::commit::run(),
        SubCommand::Sync => commands::sync::run(),
        SubCommand::Branch(args) => commands::branch::run(args),
        SubCommand::Undo => commands::undo::run(),
        SubCommand::Config => commands::config::run(),
    }
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
        "Exit".to_string(),
    ];

    let choice = Select::new("Select a command to run (Esc to Exit):", options)
        .with_render_config(get_theme())
        .prompt()?;
    
    let cmd_str = choice.split_whitespace().next().unwrap_or("");

    match cmd_str {
        "Init" => Ok(SubCommand::Init),
        "Add" => Ok(SubCommand::Add),
        "Commit" => Ok(SubCommand::Commit),
        "Sync" => Ok(SubCommand::Sync),
        "Branch" => {
            let actions = vec![
                format!("{:<7} {}", "Switch", "- Switch to another branch".bright_black()),
                format!("{:<7} {}", "Delete", "- Delete a branch".bright_black()),
                format!("{:<7} {}", "Start", "- Create a new feature/hotfix branch".bright_black()),
                format!("{:<7} {}", "Finish", "- Merge current branch into develop/main".bright_black()),
            ];
            let action_choice = Select::new("Select branch action (Esc to Back):", actions)
                .with_render_config(get_theme())
                .prompt()?;
            
            let action_str = action_choice.split_whitespace().next().unwrap_or("");
            
            let action = match action_str {
                "Switch" => BranchAction::Switch,
                "Delete" => BranchAction::Delete,
                "Start" => BranchAction::Start,
                "Finish" => BranchAction::Finish,
                _ => anyhow::bail!("Invalid branch action selected"),
            };
            
            Ok(SubCommand::Branch(BranchArgs { action }))
        },
        "Undo" => Ok(SubCommand::Undo),
        "Config" => Ok(SubCommand::Config),
        "Exit" => anyhow::bail!(InquireError::OperationCanceled),
        _ => anyhow::bail!("Invalid command selected"),
    }
}