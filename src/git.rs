use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct GitOutput {
    pub stdout: String,
}

#[derive(Clone, Debug)]
pub struct StatusEntry {
    pub index_status: char,
    pub worktree_status: char,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct RemoteEntry {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct ReflogEntry {
    pub hash: String,
    pub summary: String,
}

static GIT_STEP_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn next_step_number() -> usize {
    GIT_STEP_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn infer_git_purpose(args: &[&str]) -> String {
    if args.is_empty() {
        return "Run git command requested by the workflow.".to_string();
    }

    if args == ["branch", "--show-current"] {
        return "Detect current branch to decide follow-up actions.".to_string();
    }
    if args == ["status", "--porcelain"] {
        return "Read concise working tree status for automation checks.".to_string();
    }
    if args == ["diff", "--cached", "--name-only"] {
        return "List staged files before creating a commit.".to_string();
    }
    if args.len() >= 2 && args[0] == "checkout" && args[1] == "-b" {
        return "Create a new branch from current HEAD and switch to it.".to_string();
    }
    if args.len() >= 2 && args[0] == "checkout" {
        return "Switch working branch to the selected target branch.".to_string();
    }
    if args.len() >= 2 && args[0] == "merge" && args[1] == "--squash" {
        return "Collect branch changes as one combined commit.".to_string();
    }
    if args.len() >= 2 && args[0] == "commit" && args[1] == "-m" {
        return "Create a commit with provided commit message.".to_string();
    }
    if args.len() >= 2 && args[0] == "commit" && args[1] == "-F" {
        return "Create a commit from a prepared message file.".to_string();
    }
    if args.len() >= 2 && args[0] == "branch" && args[1] == "-d" {
        return "Delete local branch that has been merged already.".to_string();
    }
    if args.len() >= 2 && args[0] == "branch" && args[1] == "-D" {
        return "Force delete local branch even when not fully merged.".to_string();
    }
    if args.len() >= 2 && args[0] == "branch" && args[1] == "-m" {
        return "Rename branch to align with repository conventions.".to_string();
    }
    if args.len() >= 3 && args[0] == "symbolic-ref" && args[1] == "HEAD" {
        return "Point HEAD to target branch reference (including unborn branch).".to_string();
    }
    if args.len() >= 2 && args[0] == "add" {
        return "Stage selected file changes for the next commit.".to_string();
    }
    if args.len() >= 2 && args[0] == "stash" && args[1] == "push" {
        return "Temporarily save uncommitted changes before sync operations.".to_string();
    }
    if args.len() >= 2 && args[0] == "stash" && args[1] == "pop" {
        return "Restore previously stashed local changes.".to_string();
    }
    if args.len() >= 2 && args[0] == "pull" && args[1] == "--rebase" {
        return "Fetch remote updates and replay local commits on top.".to_string();
    }
    if args.len() >= 2 && args[0] == "push" && args[1] == "-u" {
        return "Push branch and set upstream tracking for future sync.".to_string();
    }
    if args.len() >= 3 && args[0] == "push" && args[2] == "--delete" {
        return "Delete branch from selected remote repository.".to_string();
    }
    if args[0] == "push" {
        return "Push local commits to tracked remote branch.".to_string();
    }
    if args.len() >= 2 && args[0] == "rebase" && args[1] == "--abort" {
        return "Cancel current rebase and return to previous state.".to_string();
    }
    if args.len() >= 2 && args[0] == "merge" && args[1] == "--abort" {
        return "Cancel current merge and restore pre-merge state.".to_string();
    }
    if args.len() >= 2 && args[0] == "reset" && args[1] == "--hard" {
        return "Reset index and working tree to target commit (discard local changes)."
            .to_string();
    }
    if args.len() >= 2 && args[0] == "reset" && args[1] == "--soft" {
        return "Move branch pointer while keeping changes staged.".to_string();
    }
    if args.len() >= 2 && args[0] == "log" && args[1] == "--oneline" {
        return "Show recent commit history for rollback selection.".to_string();
    }
    if args.len() >= 2 && args[0] == "reflog" {
        return "Show branch movement history for operation-based undo.".to_string();
    }
    if args.len() >= 2 && args[0] == "tag" && args[1] == "--list" {
        return "Check whether target tag already exists.".to_string();
    }
    if args[0] == "tag" {
        return "Create a release tag for current commit.".to_string();
    }
    if args.len() >= 2 && args[0] == "describe" && args[1] == "--tags" {
        return "Find latest reachable release tag.".to_string();
    }
    if args == ["remote"] {
        return "List configured remote aliases.".to_string();
    }
    if args.len() >= 3 && args[0] == "remote" && args[1] == "add" {
        return "Add a new remote alias and URL.".to_string();
    }
    if args.len() >= 3 && args[0] == "ls-remote" && args[1] == "--heads" {
        return "Check whether branch exists on selected remote.".to_string();
    }
    if args.len() >= 3 && args[0] == "config" && args[1] == "--get" {
        return "Read repository configuration value.".to_string();
    }
    if args.len() >= 3 && args[0] == "rev-parse" && args[1] == "--verify" && args[2] == "HEAD" {
        return "Check whether repository already has at least one commit.".to_string();
    }
    if args.len() >= 4
        && args[0] == "rev-parse"
        && args[1] == "--abbrev-ref"
        && args[2] == "--symbolic-full-name"
        && args[3] == "@{upstream}"
    {
        return "Resolve upstream branch that current branch tracks.".to_string();
    }
    if args.len() >= 3 && args[0] == "init" && args[1] == "-b" {
        return "Initialize a new repository and set default branch name.".to_string();
    }
    if args.len() >= 3 && args[0] == "commit" && args[1] == "--allow-empty" {
        return "Create initial placeholder commit for repository bootstrap.".to_string();
    }
    if args.len() >= 3 && args[0] == "branch" && args[1] == "--list" {
        return "Check whether specific local branch exists.".to_string();
    }
    if args.len() >= 3 && args[0] == "branch" && args[1] == "--format" {
        return "List local branch names for selection.".to_string();
    }
    if args.len() >= 3 && args[0] == "check-ref-format" && args[1] == "--branch" {
        return "Validate branch name against Git reference rules.".to_string();
    }

    format!(
        "Run git subcommand `{}` as part of the current workflow.",
        args[0]
    )
}

fn print_block(label: &str, text: &str, is_stderr: bool) {
    let styled_label = format!("{label}:").truecolor(125, 142, 149);
    if is_stderr {
        eprintln!("{styled_label}");
        for line in text.lines() {
            eprintln!("{}", format!("  {line}").truecolor(125, 142, 149));
        }
    } else {
        println!("{styled_label}");
        for line in text.lines() {
            println!("{}", format!("  {line}").truecolor(251, 224, 195));
        }
    }
}

fn execute_git(args: &[&str], cwd: &Path, allow_fail: bool) -> Result<(bool, GitOutput)> {
    let command_preview = format!("git {}", args.join(" "));
    let step = next_step_number();
    let purpose = infer_git_purpose(args);
    println!(
        "{}",
        format!("[Step {step}] {}", purpose).truecolor(255, 187, 152)
    );
    println!(
        "{}",
        format!("Command: $ {command_preview}").truecolor(255, 187, 152)
    );

    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run git command: $ {command_preview}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim_end_matches(&['\r', '\n'][..])
        .to_string();
    let stderr = String::from_utf8_lossy(&output.stderr)
        .trim_end_matches(&['\r', '\n'][..])
        .to_string();

    if !stdout.is_empty() {
        print_block("Output", &stdout, false);
    }
    if !stderr.is_empty() {
        print_block("Git notes", &stderr, true);
    }

    if output.status.success() {
        println!("{}", "Result: success".truecolor(125, 142, 149));
    } else {
        eprintln!("{}", "Result: failed".truecolor(125, 142, 149));
    }

    if !output.status.success() && !allow_fail {
        return Err(anyhow!("git command failed: $ {command_preview}"));
    }

    Ok((output.status.success(), GitOutput { stdout }))
}

pub fn run_git(args: &[&str]) -> Result<GitOutput> {
    run_git_in_dir(args, Path::new("."))
}

pub fn run_git_in_dir(args: &[&str], cwd: &Path) -> Result<GitOutput> {
    let (_, out) = execute_git(args, cwd, false)?;
    Ok(out)
}

pub fn run_git_allow_fail_in_dir(args: &[&str], cwd: &Path) -> Result<(bool, GitOutput)> {
    execute_git(args, cwd, true)
}

pub fn is_git_repo(cwd: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(cwd)
        .output()
        .context("failed to check git repository status")?;

    if !output.status.success() {
        return Ok(false);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.trim() == "true")
}

pub fn current_branch(cwd: &Path) -> Result<String> {
    let output = run_git_in_dir(&["branch", "--show-current"], cwd)?;
    Ok(output.stdout)
}

pub fn branch_exists(cwd: &Path, branch: &str) -> Result<bool> {
    let output = run_git_in_dir(&["branch", "--list", branch], cwd)?;
    if !output.stdout.trim().is_empty() {
        return Ok(true);
    }

    // In an unborn repository, `git branch --list <name>` can be empty even
    // when HEAD is already bound to that branch (e.g. refs/heads/main).
    let current = current_branch(cwd)?;
    Ok(current.trim() == branch)
}

pub fn is_valid_branch_ref(cwd: &Path, branch: &str) -> Result<bool> {
    let (ok, _) = run_git_allow_fail_in_dir(&["check-ref-format", "--branch", branch], cwd)?;
    Ok(ok)
}

pub fn list_local_branches(cwd: &Path) -> Result<Vec<String>> {
    let output = run_git_in_dir(&["branch", "--format", "%(refname:short)"], cwd)?;
    let branches = output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();
    Ok(branches)
}

pub fn staged_files(cwd: &Path) -> Result<Vec<String>> {
    let output = run_git_in_dir(&["diff", "--cached", "--name-only"], cwd)?;
    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

pub fn working_tree_entries(cwd: &Path) -> Result<Vec<StatusEntry>> {
    let output = run_git_in_dir(&["status", "--porcelain"], cwd)?;
    let mut entries = Vec::new();

    for line in output.stdout.lines().filter(|line| !line.is_empty()) {
        let bytes = line.as_bytes();
        let (index_status, worktree_status, path_start) = if bytes.len() >= 3 && bytes[2] == b' ' {
            (bytes[0] as char, bytes[1] as char, 3usize)
        } else if bytes.len() >= 2 && bytes[1] == b' ' {
            // Fallback for non-standard one-status lines.
            (bytes[0] as char, ' ', 2usize)
        } else {
            continue;
        };
        let path = line[path_start..].trim().to_string();
        entries.push(StatusEntry {
            index_status,
            worktree_status,
            path,
        });
    }

    Ok(entries)
}

pub fn stageable_files(cwd: &Path) -> Result<Vec<String>> {
    let mut files: Vec<String> = working_tree_entries(cwd)?
        .into_iter()
        .filter(|entry| entry.index_status != ' ' || entry.worktree_status != ' ')
        .map(|entry| entry.path)
        .collect();
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn is_clean_worktree(cwd: &Path) -> Result<bool> {
    Ok(working_tree_entries(cwd)?.is_empty())
}

pub fn has_staged_changes(cwd: &Path) -> Result<bool> {
    Ok(!staged_files(cwd)?.is_empty())
}

/// Paths that are unmerged (e.g. after merge/rebase conflict).
pub fn unmerged_files(cwd: &Path) -> Result<Vec<String>> {
    let entries = working_tree_entries(cwd)?;
    Ok(entries
        .into_iter()
        .filter(|e| e.index_status == 'U' || e.worktree_status == 'U')
        .map(|e| e.path)
        .collect())
}

/// Files in the working tree that still contain conflict markers.
pub fn files_with_conflict_markers(cwd: &Path) -> Result<Vec<String>> {
    use std::fs;
    let paths = stageable_files(cwd)?;
    let mut with_markers = Vec::new();
    for path in paths {
        let full = cwd.join(&path);
        let Ok(content) = fs::read_to_string(&full) else { continue };
        if content.contains("<<<<<<<") {
            with_markers.push(path);
        }
    }
    Ok(with_markers)
}

pub fn add_all(cwd: &Path) -> Result<()> {
    run_git_in_dir(&["add", "-A"], cwd)?;
    Ok(())
}

pub fn upstream_branch(cwd: &Path) -> Result<Option<String>> {
    let (ok, output) = run_git_allow_fail_in_dir(
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
        cwd,
    )?;
    if !ok || output.stdout.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(output.stdout))
}

/// Parses upstream ref (e.g. `refs/remotes/origin/main` or `origin/main`) into `(remote, branch)`.
pub fn upstream_remote_and_branch(cwd: &Path) -> Result<Option<(String, String)>> {
    let raw = match upstream_branch(cwd)? {
        Some(s) => s,
        None => return Ok(None),
    };
    let s = raw.trim();
    let rest = s
        .strip_prefix("refs/remotes/")
        .unwrap_or(s);
    let Some((remote, branch)) = rest.split_once('/') else {
        return Ok(None);
    };
    if remote.is_empty() || branch.is_empty() {
        return Ok(None);
    }
    Ok(Some((remote.to_string(), branch.to_string())))
}

pub fn default_branch(cwd: &Path) -> Result<String> {
    if branch_exists(cwd, "main")? {
        return Ok("main".to_string());
    }
    if branch_exists(cwd, "master")? {
        return Ok("master".to_string());
    }

    let branches = list_local_branches(cwd)?;
    branches
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no local branches found; create first commit and branch first"))
}

pub fn detect_parent_branch(cwd: &Path, current: &str) -> Result<String> {
    if let Some(upstream) = upstream_branch(cwd)? {
        let upstream_short = upstream
            .split_once('/')
            .map_or_else(|| upstream.clone(), |(_, right)| right.to_string());
        if upstream_short != current && branch_exists(cwd, &upstream_short)? {
            return Ok(upstream_short);
        }
    }

    let default = default_branch(cwd)?;
    if default != current {
        return Ok(default);
    }

    let mut branches = list_local_branches(cwd)?;
    branches.retain(|branch| branch != current);
    branches
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("cannot find parent branch for {current}"))
}

pub fn checkout_branch(cwd: &Path, branch: &str) -> Result<()> {
    run_git_in_dir(&["checkout", branch], cwd)?;
    Ok(())
}

pub fn squash_merge_branch(cwd: &Path, branch: &str) -> Result<()> {
    run_git_in_dir(&["merge", "--squash", branch], cwd)?;
    Ok(())
}

pub fn commit_with_message(cwd: &Path, message: &str) -> Result<()> {
    run_git_in_dir(&["commit", "-m", message], cwd)?;
    Ok(())
}

pub fn delete_branch_force(cwd: &Path, branch: &str) -> Result<()> {
    run_git_in_dir(&["branch", "-D", branch], cwd)?;
    Ok(())
}

pub fn try_delete_branch(cwd: &Path, branch: &str, force: bool) -> Result<bool> {
    let args = if force {
        vec!["branch", "-D", branch]
    } else {
        vec!["branch", "-d", branch]
    };
    let (ok, _) = run_git_allow_fail_in_dir(&args, cwd)?;
    Ok(ok)
}

pub fn origin_remote_url(cwd: &Path) -> Result<Option<String>> {
    let (ok, output) = run_git_allow_fail_in_dir(&["config", "--get", "remote.origin.url"], cwd)?;
    if !ok || output.stdout.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(output.stdout))
}

pub fn github_repo_slug_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        return Some(rest.trim_end_matches(".git").to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        return Some(rest.trim_end_matches(".git").to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("http://github.com/") {
        return Some(rest.trim_end_matches(".git").to_string());
    }
    None
}

pub fn list_remotes(cwd: &Path) -> Result<Vec<RemoteEntry>> {
    let names_out = run_git_in_dir(&["remote"], cwd)?;
    let mut remotes = Vec::new();
    for name in names_out
        .stdout
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let key = format!("remote.{name}.url");
        let url_out = run_git_in_dir(&["config", "--get", &key], cwd)?;
        if !url_out.stdout.trim().is_empty() {
            remotes.push(RemoteEntry {
                name: name.to_string(),
                url: url_out.stdout.trim().to_string(),
            });
        }
    }
    Ok(remotes)
}

pub fn remote_exists(cwd: &Path, name: &str) -> Result<bool> {
    Ok(list_remotes(cwd)?.iter().any(|remote| remote.name == name))
}

pub fn add_remote(cwd: &Path, name: &str, url: &str) -> Result<()> {
    run_git_in_dir(&["remote", "add", name, url], cwd)?;
    Ok(())
}

pub fn remote_branch_exists(cwd: &Path, remote: &str, branch: &str) -> Result<bool> {
    let output = run_git_in_dir(&["ls-remote", "--heads", remote, branch], cwd)?;
    Ok(!output.stdout.trim().is_empty())
}

pub fn delete_remote_branch(cwd: &Path, remote: &str, branch: &str) -> Result<()> {
    run_git_in_dir(&["push", remote, "--delete", branch], cwd)?;
    Ok(())
}

pub fn has_uncommitted_changes(cwd: &Path) -> Result<bool> {
    Ok(!working_tree_entries(cwd)?.is_empty())
}

pub fn stash_push(cwd: &Path, message: &str) -> Result<bool> {
    let output = run_git_in_dir(&["stash", "push", "-u", "-m", message], cwd)?;
    let text = output.stdout.to_lowercase();
    Ok(!text.contains("no local changes to save"))
}

pub fn stash_pop(cwd: &Path) -> Result<()> {
    run_git_in_dir(&["stash", "pop"], cwd)?;
    Ok(())
}

pub fn pull_rebase(cwd: &Path, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
    let mut args = vec!["pull", "--rebase"];
    if let Some(r) = remote {
        args.push(r);
    }
    if let Some(b) = branch {
        args.push(b);
    }
    run_git_in_dir(&args, cwd)?;
    Ok(())
}

pub fn push_current(cwd: &Path, set_upstream_remote: Option<&str>, branch: &str) -> Result<()> {
    if let Some(remote) = set_upstream_remote {
        run_git_in_dir(&["push", "-u", remote, branch], cwd)?;
    } else {
        run_git_in_dir(&["push"], cwd)?;
    }
    Ok(())
}

pub fn rebase_abort(cwd: &Path) -> Result<()> {
    run_git_in_dir(&["rebase", "--abort"], cwd)?;
    Ok(())
}

pub fn merge_abort(cwd: &Path) -> Result<()> {
    run_git_in_dir(&["merge", "--abort"], cwd)?;
    Ok(())
}

pub fn reset_hard_head(cwd: &Path) -> Result<()> {
    run_git_in_dir(&["reset", "--hard", "HEAD"], cwd)?;
    Ok(())
}

pub fn list_recent_commits(cwd: &Path, limit: usize) -> Result<Vec<String>> {
    let limit_str = limit.to_string();
    let output = run_git_in_dir(&["log", "--oneline", "-n", &limit_str], cwd)?;
    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

pub fn list_reflog(cwd: &Path, limit: usize) -> Result<Vec<ReflogEntry>> {
    let limit_str = limit.to_string();
    let output = run_git_in_dir(&["reflog", "--pretty=%h\t%gs", "-n", &limit_str], cwd)?;
    let mut entries = Vec::new();
    for line in output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
    {
        let (hash, summary) = if let Some((h, s)) = line.split_once('\t') {
            (h.trim(), s.trim())
        } else if let Some((h, s)) = line.split_once(' ') {
            (h.trim(), s.trim())
        } else {
            continue;
        };
        if hash.is_empty() {
            continue;
        }
        entries.push(ReflogEntry {
            hash: hash.to_string(),
            summary: summary.to_string(),
        });
    }
    Ok(entries)
}

pub fn reset_to(cwd: &Path, target: &str, hard: bool) -> Result<()> {
    if hard {
        run_git_in_dir(&["reset", "--hard", target], cwd)?;
    } else {
        run_git_in_dir(&["reset", "--soft", target], cwd)?;
    }
    Ok(())
}

pub fn latest_tag(cwd: &Path) -> Result<Option<String>> {
    let (ok, output) = run_git_allow_fail_in_dir(&["describe", "--tags", "--abbrev=0"], cwd)?;
    if !ok || output.stdout.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(output.stdout.trim().to_string()))
}

pub fn tag_exists(cwd: &Path, tag: &str) -> Result<bool> {
    let output = run_git_in_dir(&["tag", "--list", tag], cwd)?;
    Ok(!output.stdout.trim().is_empty())
}

pub fn create_tag(cwd: &Path, tag: &str) -> Result<()> {
    run_git_in_dir(&["tag", tag], cwd)?;
    Ok(())
}

pub fn has_commits(cwd: &Path) -> Result<bool> {
    let (ok, _) = run_git_allow_fail_in_dir(&["rev-parse", "--verify", "HEAD"], cwd)?;
    Ok(ok)
}

pub fn create_empty_commit(cwd: &Path, message: &str) -> Result<()> {
    run_git_in_dir(&["commit", "--allow-empty", "-m", message], cwd)?;
    Ok(())
}
