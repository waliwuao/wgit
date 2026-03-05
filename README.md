# wgit

wgit is a guided Git assistant for beginners.
It reduces command memorization with interactive selections and enforces safer workflows.

## Features

- Core workflow implemented: `init`, `add`, `commit`, `delete`, `start`, `switch`, `finish`, `remote`, `undo`, `sync`, `menu`, `update`
- `add`: parses `git status --porcelain`, supports searchable multi-select staging
- `commit`: staged check + commit type selection + structured commit editor
- `delete`: guided local branch deletion with safe/force mode and optional remote cleanup
- `start`: guided branch type selection and branch name validation
- `switch`: searchable local branch list with dirty worktree warning
- `finish`: detect parent branch, squash merge with conflict options, guided merge message, optional release tag on main, and optional remote branch cleanup
- `remote`: detect remotes and add remote aliases interactively
- `undo`: reset by commit or reflog operation with soft/hard mode
- `sync`: auto-stash, pull --rebase, push, and restore stash
- `update`: GitHub Releases latest-version detection and binary self-replacement
- Unified Git command runner with colored command preview and contextual errors
- Reusable TUI primitives for single select, multi select, and text input/editor

## Quick Start

```bash
cargo run -- menu
```

Or run commands directly:

```bash
cargo run -- add
cargo run -- commit
cargo run -- start
```

Run help:

```bash
cargo run -- --help
```

## Project Structure

- `src/main.rs`: application entrypoint and error boundary
- `src/cli.rs`: CLI parser and command declarations
- `src/commands/`: command handlers and dispatcher
- `src/git.rs`: Git command execution facade
- `src/config.rs`: local config bootstrap (`.git/wgit.toml`)
- `src/utils.rs`: reusable interactive prompt helpers

## Optimization Docs

- `docs/github-flow-gap-analysis.md`: current capability vs GitHub Flow gap analysis
- `docs/high-risk-command-safety-spec.md`: high-risk command safety and recovery rules
- `docs/convention-normalization-spec.md`: branch/commit/tag convention normalization
- `docs/quality-gates-and-test-matrix.md`: minimal test matrix and CI gate recommendations
- `docs/beginner-github-flow-playbook.md`: scenario-based beginner workflow handbook

## Development

```bash
cargo check
cargo fmt
```

## Notes

- `update` resolves release repository from `git remote origin`; ensure it points to GitHub.
