use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wgit", version, about = "A customized git flow wrapper")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<SubCommand>,
}

#[derive(Subcommand, Debug)]
pub enum SubCommand {
    Add,
    Init,
    Sync,
    Undo,
    Exit,
    Config,
    Branch(BranchArgs),
    Commit,
    Update,
}

#[derive(Parser, Debug)]
pub struct BranchArgs {
    #[command(subcommand)]
    pub action: Option<BranchAction>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum BranchAction {
    Start,
    Delete,
    Switch,
    Finish,
}