mod cli;
mod config;
mod git;
mod utils;
mod commands;

use clap::Parser;
use cli::{Cli, SubCommand};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
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