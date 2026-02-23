use crate::git;
use crate::config::{load_config, save_config, ReviewMode};
use inquire::{Select, Text};

pub fn run() -> anyhow::Result<()> {
    let mut config = load_config()?;

    let options = vec![
        "Add remote repo", 
        "Set review mode (Local Merge vs Remote Review)", 
        "Exit"
    ];

    loop {
        let choice = Select::new("Wgit Config Menu:", options.clone()).prompt()?;
        
        match choice {
            "Add remote repo" => {
                let name = Text::new("Remote name (e.g., origin):").prompt()?;
                let url = Text::new("Remote URL:").prompt()?;
                
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
                let m = Select::new("Select review mode:", modes).prompt()?;
                
                if m.starts_with("LocalMerge") {
                    config.review_mode = ReviewMode::LocalMerge;
                } else {
                    config.review_mode = ReviewMode::RemoteReview;
                }
                save_config(&config)?;
                println!("Review mode updated successfully.");
            }
            _ => break,
        }
    }
    Ok(())
}