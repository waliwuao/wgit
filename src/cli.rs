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
    Context,
    Fops(FopsArgs),
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

#[derive(Parser, Debug)]
pub struct FopsArgs {
    #[command(subcommand)]
    pub action: Option<FopsAction>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum FopsAction {
    Copy { src: String, dest: String },
    Move { src: String, dest: String },
    Remove { path: String },
    Rename { src: String, dest: String },
    Chmod { path: Option<String> },
    Size { path: String },
    Netinfo,
}