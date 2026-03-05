use crate::{git, utils};
use anyhow::{Result, bail};
use std::path::Path;

pub fn run() -> Result<()> {
    let cwd = Path::new(".");
    let remotes = git::list_remotes(cwd)?;
    if remotes.is_empty() {
        println!("No remote repositories detected.");
    } else {
        println!("Detected remote repositories:");
        for remote in &remotes {
            println!("  - {} -> {}", remote.name, remote.url);
        }
    }

    let should_add = utils::confirm("Add a new remote alias?")?;
    if !should_add {
        println!("Remote command completed without changes.");
        return Ok(());
    }

    let name = utils::input_text("Remote alias (example: origin)")?;
    let name = name.trim();
    if name.is_empty() {
        println!("Remote creation canceled: empty alias.");
        return Ok(());
    }
    if name.contains(' ') {
        bail!("invalid remote alias: spaces are not allowed");
    }
    if git::remote_exists(cwd, name)? {
        bail!("remote alias already exists: {name}");
    }

    let url = utils::input_text("Remote URL")?;
    let url = url.trim();
    if url.is_empty() {
        println!("Remote creation canceled: empty URL.");
        return Ok(());
    }

    git::add_remote(cwd, name, url)?;
    println!("Remote `{name}` added successfully.");
    Ok(())
}
