use crate::git;

pub fn run() -> anyhow::Result<()> {
    git::run_git(&["init"])?;

    if git::get_output(&["rev-parse", "HEAD"]).is_err() {
        git::run_git(&["commit", "--allow-empty", "-m", "chore: initial wgit commit"])?;
    }

    git::run_git(&["branch", "-M", "main"])?;
    
    let branches = git::get_output(&["branch", "--format=%(refname:short)"])?;
    if !branches.lines().any(|b| b.trim() == "develop") {
        git::run_git(&["branch", "develop"])?;
    }
    git::run_git(&["checkout", "develop"])?;

    install_hook()?;

    println!("wgit initialized. 'main' and 'develop' branches are protected.");
    Ok(())
}

fn install_hook() -> anyhow::Result<()> {
    let hook_path = std::path::PathBuf::from(".git/hooks/pre-commit");
    let hook_script = r#"#!/bin/sh
branch="$(git rev-parse --abbrev-ref HEAD)"
if [ "$branch" = "main" ] || [ "$branch" = "develop" ]; then
    echo "wgit Error: Direct commits to $branch are forbidden."
    exit 1
fi
"#;
    std::fs::write(&hook_path, hook_script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }
    Ok(())
}