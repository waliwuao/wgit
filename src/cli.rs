use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "wgit", version, about = "A guided Git assistant for beginners")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Initialize wgit in the current repository.
    Init,
    /// Stage files with a multi-select interface.
    Add,
    /// Commit changes with guided prompts.
    Commit,
    /// Create a new branch from the current branch.
    Start,
    /// Finish current branch and merge into parent.
    Finish,
    /// Manage remote repositories.
    Remote,
    /// Switch to another branch.
    Switch,
    /// Roll back with selected strategy and level.
    Undo,
    /// Pull and push with assisted flow.
    Sync,
    /// Update wgit to the latest release.
    Update,
    /// Open command menu.
    Menu,
}

pub fn parse() -> Cli {
    Cli::parse()
}
