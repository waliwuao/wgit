use crate::git;
use crate::config::load_config;
use colored::Colorize;
use std::fs;
use std::io::Read;

pub fn run() -> anyhow::Result<()> {
    let config = load_config()?;
    
    println!("\n{}", "── Initializing Git Flow ───────────────────────────────────────────────".cyan());

    println!("{} Creating repository...", " 1/4 ".on_cyan().black());
    git::run_git(&["init", &format!("--initial-branch={}", config.main_branch)])?;

    if git::get_output(&["rev-parse", "HEAD"]).is_err() {
        println!("{} Creating initial commit...", " 2/4 ".on_cyan().black());
        git::run_git(&["commit", "--allow-empty", "-m", "chore: initial wgit commit"])?;
    }

    println!("{} Setting up branch structure...", " 3/4 ".on_cyan().black());
    git::run_git(&["branch", "-M", &config.main_branch])?;
    let branches = git::get_output(&["branch", "--format=%(refname:short)"])?;
    if !branches.lines().any(|b| b.trim() == config.dev_branch) {
        git::run_git(&["branch", &config.dev_branch])?;
    }
    git::run_git(&["checkout", &config.dev_branch])?;

    println!("{} Installing protection hooks...", " 4/4 ".on_cyan().black());
    install_hook(&config.main_branch, &config.dev_branch)?;

    println!("{}\n", "wgit flow initialized successfully!".green().bold());
    Ok(())
}

pub fn install_hook(main: &str, dev: &str) -> anyhow::Result<()> {
    let hook_path = std::path::PathBuf::from(".git/hooks/pre-commit");
    let marker_start = "# --- WGIT-HOOK-BEGIN ---";
    let marker_end = "# --- WGIT-HOOK-END ---";
    
    let hook_payload = format!(
"{}\nbranch=\"$(git rev-parse --abbrev-ref HEAD)\"\nif [ \"$branch\" = \"{}\" ] || [ \"$branch\" = \"{}\" ]; then\n    echo \"wgit Error: Direct commits to $branch are forbidden.\"\n    exit 1\nfi\n{}", 
    marker_start, main, dev, marker_end);

    let mut current_content = String::new();
    if hook_path.exists() {
        fs::File::open(&hook_path)?.read_to_string(&mut current_content)?;
    }

    let new_content = if let (Some(s), Some(e)) = (current_content.find(marker_start), current_content.find(marker_end)) {
        let mut content = current_content.clone();
        content.replace_range(s..e + marker_end.len(), &hook_payload);
        content
    } else {
        let base = if current_content.is_empty() {
            "#!/bin/sh".to_string()
        } else {
            current_content
        };
        format!("{}\n\n{}", base, hook_payload)
    };

    fs::write(&hook_path, new_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }
    Ok(())
}