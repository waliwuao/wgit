use anyhow::Result;
use colored::Colorize;
use ignore::WalkBuilder;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
pub struct ProjectTemplate {
    pub template_name: String,
    pub description: String,
    pub files: BTreeMap<String, String>,
}

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    println!("{}", "── Generating Context ──────────────────────────────────────────────────".cyan());
    println!("Scanning files in: {}", current_dir.display());

    let mut allowed_paths = HashSet::new();
    let walker = WalkBuilder::new(&current_dir)
        .git_ignore(true)
        .hidden(false)
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "info" && name != "target" && name != "node_modules" && name != ".idea" && name != ".vscode"
        })
        .build();

    for result in walker {
        if let Ok(entry) = result {
            if entry.path() != current_dir {
                allowed_paths.insert(entry.path().to_path_buf());
            }
        }
    }

    let mut tree_output = String::new();
    let mut content_output = String::new();
    let mut template_files = BTreeMap::new();
    let mut total_chars = 0;

    tree_output.push_str(&format!("{}/\n", project_name));
    render_tree(&current_dir, &allowed_paths, "", &mut tree_output)?;

    let mut sorted_paths: Vec<_> = allowed_paths.iter().collect();
    sorted_paths.sort();

    for path in sorted_paths {
        if path.is_file() {
            let relative_path = path
                .strip_prefix(&current_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Ok(content) = fs::read_to_string(path) {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                content_output.push_str(&format!("\n### {}\n", relative_path));
                content_output.push_str(&format!("```{}\n", ext));
                content_output.push_str(&content);
                content_output.push_str("\n```\n");
                
                total_chars += content.len();
                template_files.insert(relative_path, content);
            }
        } else if path.is_dir() {
             let relative_path = path
                .strip_prefix(&current_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
             let dir_key = if relative_path.ends_with('/') { relative_path } else { format!("{}/", relative_path) };
             template_files.insert(dir_key, "".to_string());
        }
    }

    let info_dir = current_dir.join("info");
    if !info_dir.exists() {
        fs::create_dir_all(&info_dir)?;
    }

    let final_markdown = format!(
        "# Project Context: {}\n\n## File Structure\n\n```text\n{}\n```\n\n## File Contents\n{}",
        project_name, tree_output, content_output
    );
    
    let md_path = info_dir.join("context.md");
    fs::write(&md_path, &final_markdown)?;

    let template = ProjectTemplate {
        template_name: project_name.clone(),
        description: format!("Context snapshot of {}", project_name),
        files: template_files,
    };
    
    let json_content = serde_json::to_string_pretty(&template)?;
    let json_path = info_dir.join(format!("{}_template.json", project_name));
    fs::write(&json_path, json_content)?;

    update_gitignore(&current_dir)?;

    println!("Context generated successfully!");
    println!("  - Markdown: {}", format!("{:?}", md_path).green());
    println!("  - JSON:     {}", format!("{:?}", json_path).green());
    
    let estimated_tokens = total_chars / 4;
    println!("  - Tokens:   ~{} (Estimated)", estimated_tokens.to_string().yellow());

    Ok(())
}

fn render_tree(
    current_dir: &Path,
    allowed: &HashSet<PathBuf>,
    prefix: &str,
    output: &mut String,
) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(current_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| allowed.contains(&e.path()))
        .collect();

    entries.sort_by(|a, b| {
        let a_name = a.file_name().to_string_lossy().to_string();
        let b_name = b.file_name().to_string_lossy().to_string();
        a_name.cmp(&b_name)
    });

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = path.is_dir();

        let connector = if is_last { "└── " } else { "├── " };
        let display_name = if is_dir { format!("{}/", name) } else { name };

        output.push_str(&format!("{}{}{}\n", prefix, connector, display_name));

        if is_dir {
            let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            render_tree(&path, allowed, &child_prefix, output)?;
        }
    }

    Ok(())
}

fn update_gitignore(root: &Path) -> Result<()> {
    let gitignore_path = root.join(".gitignore");
    let entry = "info/";
    
    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if !content.lines().any(|line| line.trim() == "info/" || line.trim() == "info") {
            use std::io::Write;
            let mut file = fs::OpenOptions::new().append(true).open(&gitignore_path)?;
            writeln!(file, "\n{}", entry)?;
            println!("Updated .gitignore: Added 'info/'");
        }
    } else {
        fs::write(&gitignore_path, format!("{}\n", entry))?;
        println!("Created .gitignore: Added 'info/'");
    }
    Ok(())
}