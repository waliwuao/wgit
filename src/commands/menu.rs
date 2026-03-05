use crate::cli::Command;
use crate::utils;
use anyhow::Result;

pub fn run() -> Result<Option<Command>> {
    let labels = vec![
        "init   - initialize repository and wgit config".to_string(),
        "add    - stage files with multi-select".to_string(),
        "commit - create structured commit message".to_string(),
        "start  - create and switch to new branch".to_string(),
        "finish - squash merge current branch".to_string(),
        "remote - detect and add remote aliases".to_string(),
        "switch - switch local branch".to_string(),
        "undo   - rollback by commit or operation".to_string(),
        "sync   - auto stash + pull/push assistant".to_string(),
        "update - self-update from GitHub release".to_string(),
        "exit   - close menu".to_string(),
    ];

    let selected = utils::select_one("Select a command", &labels)?;
    let command = match selected {
        Some(0) => Some(Command::Init),
        Some(1) => Some(Command::Add),
        Some(2) => Some(Command::Commit),
        Some(3) => Some(Command::Start),
        Some(4) => Some(Command::Finish),
        Some(5) => Some(Command::Remote),
        Some(6) => Some(Command::Switch),
        Some(7) => Some(Command::Undo),
        Some(8) => Some(Command::Sync),
        Some(9) => Some(Command::Update),
        Some(10) | None => None,
        _ => None,
    };

    Ok(command)
}
