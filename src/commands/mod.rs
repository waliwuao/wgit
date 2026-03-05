pub mod add;
pub mod commit;
pub mod delete;
pub mod finish;
pub mod init;
pub mod menu;
pub mod solve;
pub mod remote;
pub mod start;
pub mod switch;
pub mod sync;
pub mod undo;
pub mod update;

use crate::cli::Command;
use anyhow::Result;

pub fn dispatch(command: Option<Command>) -> Result<()> {
    let command = command.unwrap_or(Command::Menu);

    match command {
        Command::Init => init::run(),
        Command::Add => add::run(),
        Command::Commit => commit::run(),
        Command::Delete => delete::run(),
        Command::Start => start::run(),
        Command::Finish => finish::run(),
        Command::Solve => solve::run(),
        Command::Remote => remote::run(),
        Command::Switch => switch::run(),
        Command::Undo => undo::run(),
        Command::Sync => sync::run(),
        Command::Update => update::run(),
        Command::Menu => {
            if let Some(next) = menu::run()? {
                dispatch(Some(next))
            } else {
                Ok(())
            }
        }
    }
}
