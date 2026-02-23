use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wgit", version, about = "A customized git flow wrapper")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}

#[derive(Subcommand)]
pub enum SubCommand {
    Init,
    Add,
    Commit,
    Sync,
    Branch(BranchArgs),
    Undo,
    Config,
}

#[derive(Parser)]
pub struct BranchArgs {
    #[command(subcommand)]
    pub action: Option<BranchAction>,
}

#[derive(Subcommand, Clone)]
pub enum BranchAction {
    Switch,
    Delete,
    Start,
    Finish,
}