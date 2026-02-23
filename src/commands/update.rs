use self_update::cargo_crate_version;
use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    println!("\n{}", "── Checking for Updates ────────────────────────────────────────────────".cyan());

    let target = if cfg!(target_os = "linux") {
        "linux-amd64"
    } else if cfg!(target_os = "windows") {
        "windows-amd64"
    } else if cfg!(target_os = "macos") {
        "macos-amd64"
    } else {
        anyhow::bail!("Unsupported OS for auto-update.");
    };

    let status = self_update::backends::github::Update::configure()
        .repo_owner("waliwuao")
        .repo_name("wgit")
        .bin_name("wgit")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .target(target)
        .build()?
        .update()?;

    if status.updated() {
        println!("\n{}", format!("Successfully updated to v{}!", status.version()).green().bold());
    } else {
        println!("\n{}", "You are already using the latest version.".green());
    }

    Ok(())
}