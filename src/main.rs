mod cli;
mod commands;
mod config;
mod git;
mod utils;

use anyhow::Result;

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = cli::parse();
    commands::dispatch(cli.command)
}
