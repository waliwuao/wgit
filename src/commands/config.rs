use crate::git;
use crate::config::{load_config, save_config, ReviewMode};
use crate::utils::get_theme;
use inquire::{Select, Text};

pub fn run() -> anyhow::Result<()> {
    let mut config = load_config()?;

    let options = vec![
        "Add remote repo", 
        "Set review mode (Local Merge vs Remote Review)", 
        "Set branch names (main/develop)",
        "Exit"
    ];

    loop {
        let choice = Select::new("Wgit Config Menu:", options.clone())
            .with_render_config(get_theme())
            .prompt()?;
        
        match choice {
            "Add remote repo" => {
                let name = Text::new("Remote name (e.g., origin):")
                    .with_render_config(get_theme())
                    .prompt()?;
                let url = Text::new("Remote URL:")
                    .with_render_config(get_theme())
                    .prompt()?;
                
                let _ = git::get_output(&["remote", "add", &name, &url]);
                config.remotes.insert(name.clone(), url.clone());
                save_config(&config)?;
                println!("Remote {} added successfully.", name);
            }
            "Set review mode (Local Merge vs Remote Review)" => {
                let modes = vec![
                    "LocalMerge (Merge locally when finishing branches)", 
                    "RemoteReview (Push branch after commit for remote review)"
                ];
                let m = Select::new("Select review mode:", modes)
                    .with_render_config(get_theme())
                    .prompt()?;
                
                if m.starts_with("LocalMerge") {
                    config.review_mode = ReviewMode::LocalMerge;
                } else {
                    config.review_mode = ReviewMode::RemoteReview;
                }
                save_config(&config)?;
                println!("Review mode updated successfully.");
            }
            "Set branch names (main/develop)" => {
                let m = Text::new("Main branch name:")
                    .with_initial_value(&config.main_branch)
                    .prompt()?;
                let d = Text::new("Develop branch name:")
                    .with_initial_value(&config.dev_branch)
                    .prompt()?;
                
                config.main_branch = m;
                config.dev_branch = d;
                save_config(&config)?;
                
                crate::commands::init::install_hook(&config.main_branch, &config.dev_branch)?;
                println!("Branch names updated and hooks refreshed.");
            }
            _ => break,
        }
    }
    Ok(())
}