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

    match cli.command {
        Some(cmd) => {
            if let SubCommand::Exit = cmd {
                return Ok(());
            }
            execute_command(cmd)?
        },
        None => {
            loop {
                match interactive_select()? {
                    Some(cmd) => {
                        if let SubCommand::Exit = cmd {
                            break;
                        }
                        if let Err(e) = execute_command(cmd) {
                            println!("{} {}", "Error:".red().bold(), e);
                            println!("{}", "Press Enter to continue...".bright_black());
                            let _ = std::io::stdin().read_line(&mut String::new());
                        }
                        println!("\n");
                    },
                    None => break,
                }
            }
        }
    }

    Ok(())
}

fn execute_command(command: SubCommand) -> anyhow::Result<()> {
    match command {
        SubCommand::Add => commands::add::run(),
        SubCommand::Init => commands::init::run(),
        SubCommand::Sync => commands::sync::run(),
        SubCommand::Undo => commands::undo::run(),
        SubCommand::Config => commands::config::run(),
        SubCommand::Branch(args) => commands::branch::run(args),
        SubCommand::Commit => commands::commit::run(),
        SubCommand::Update => commands::update::run(),
        SubCommand::Context => commands::context::run(),
        SubCommand::Exit => Ok(()),
    }
}

fn interactive_select() -> anyhow::Result<Option<SubCommand>> {
    let options = vec![
        format!("{:<14} {}", "Add", "Interactively stage modified or untracked files".bright_black()),
        format!("{:<14} {}", "Init", "Initialize git-flow environment and branch protection".bright_black()),
        format!("{:<14} {}", "Sync", "Synchronize current branch with remote (Pull & Push)".bright_black()),
        format!("{:<14} {}", "Undo", "Revert changes to a specific commit or operation".bright_black()),
        format!("{:<14} {}", "Commit", "Record repository changes with structured messages".bright_black()),
        format!("{:<14} {}", "Branch", "Manage development lifecycle and branch operations".bright_black()),
        format!("{:<14} {}", "Context", "Generate project context (Markdown/JSON) for LLMs".bright_black()),
        format!("{:<14} {}", "Config", "Manage remote repositories and workflow preferences".bright_black()),
        format!("{:<14} {}", "Update", "Check for and install the latest version of wgit".bright_black()),
        format!("{:<14} {}", "Exit", "Close wgit assistant".bright_black()),
    ];

    let choice = Select::new("Wgit Assistant Menu:", options)
        .with_render_config(get_theme())
        .with_page_size(11)
        .prompt_skippable()?;
    
    let choice = match choice {
        Some(c) => c,
        None => return Ok(None),
    };

    let cmd_str = choice.split_whitespace().next().unwrap_or("");

    match cmd_str {
        "Add" => Ok(Some(SubCommand::Add)),
        "Init" => Ok(Some(SubCommand::Init)),
        "Sync" => Ok(Some(SubCommand::Sync)),
        "Undo" => Ok(Some(SubCommand::Undo)),
        "Exit" => Ok(Some(SubCommand::Exit)),
        "Config" => Ok(Some(SubCommand::Config)),
        "Branch" => Ok(Some(SubCommand::Branch(BranchArgs { action: None }))),
        "Commit" => Ok(Some(SubCommand::Commit)),
        "Update" => Ok(Some(SubCommand::Update)),
        "Context" => Ok(Some(SubCommand::Context)),
        _ => anyhow::bail!("Invalid command selected"),
    }
}