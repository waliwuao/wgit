use crate::cli::{FopsArgs, FopsAction};
use crate::utils::get_theme;
use inquire::{Select, Text, MultiSelect};
use colored::Colorize;
use std::path::Path;
use std::fs;
use anyhow::Result;

pub fn run(args: FopsArgs) -> Result<()> {
    let action = match args.action {
        Some(a) => a,
        None => interactive_select()?,
    };

    match action {
        FopsAction::Copy { paths } => {
            let (srcs, dest) = if paths.is_empty() {
                let s = prompt_multi_files("Select files/directories to copy (Space: select, Enter: confirm):")?;
                let d = Text::new("Destination directory path:").with_render_config(get_theme()).prompt()?;
                (s, d)
            } else if paths.len() < 2 {
                anyhow::bail!("Copy requires at least a source and a destination.");
            } else {
                let d = paths.last().unwrap().clone();
                let s = paths[..paths.len()-1].to_vec();
                (s, d)
            };
            copy_paths(&srcs, &dest)?;
        }
        FopsAction::Move { paths } => {
            let (srcs, dest) = if paths.is_empty() {
                let s = prompt_multi_files("Select files/directories to move (Space: select, Enter: confirm):")?;
                let d = Text::new("Destination directory path:").with_render_config(get_theme()).prompt()?;
                (s, d)
            } else if paths.len() < 2 {
                anyhow::bail!("Move requires at least a source and a destination.");
            } else {
                let d = paths.last().unwrap().clone();
                let s = paths[..paths.len()-1].to_vec();
                (s, d)
            };
            move_paths(&srcs, &dest)?;
        }
        FopsAction::Remove { paths } => {
            let targets = if paths.is_empty() {
                prompt_multi_files("Select files/directories to remove (Space: select, Enter: confirm):")?
            } else {
                paths
            };
            for p in targets { remove_path(&p)?; }
        }
        FopsAction::Rename { src, dest } => {
            let s = match src {
                Some(p) => p,
                None => prompt_single_file("Select file/directory to rename:")?
            };
            let d = match dest {
                Some(p) => p,
                None => Text::new(&format!("New name for '{}':", s)).with_render_config(get_theme()).prompt()?
            };
            rename_path(&s, &d)?;
        }
        FopsAction::Chmod { paths } => {
            let targets = if paths.is_empty() {
                prompt_multi_files("Select files/directories to change permissions (Space: select, Enter: confirm):")?
            } else {
                paths
            };
            chmod_interactive(targets)?;
        }
        FopsAction::Size { paths } => {
            let targets = if paths.is_empty() {
                prompt_multi_files("Select files/directories to check size (Space: select, Enter: confirm):")?
            } else {
                paths
            };
            for p in targets { size_path(&p)?; }
        }
        FopsAction::Netinfo => netinfo()?,
    }
    Ok(())
}

fn interactive_select() -> Result<FopsAction> {
    let actions = vec![
        format!("{:<14} {}", "Copy", "Copy file or directory (Batch supported)".bright_black()),
        format!("{:<14} {}", "Move", "Move file or directory (Batch supported)".bright_black()),
        format!("{:<14} {}", "Remove", "Delete file or directory (Batch supported)".bright_black()),
        format!("{:<14} {}", "Rename", "Rename a single file or directory".bright_black()),
        format!("{:<14} {}", "Chmod", "Change file permissions (Batch supported)".bright_black()),
        format!("{:<14} {}", "Size", "Get size of file or directory (Batch supported)".bright_black()),
        format!("{:<14} {}", "Netinfo", "Print network interfaces info".bright_black()),
    ];

    let choice = Select::new("Select fops action:", actions)
        .with_render_config(get_theme())
        .with_page_size(10)
        .prompt()?;

    let cmd_str = choice.split_whitespace().next().unwrap_or("");
    match cmd_str {
        "Copy" => Ok(FopsAction::Copy { paths: vec![] }),
        "Move" => Ok(FopsAction::Move { paths: vec![] }),
        "Remove" => Ok(FopsAction::Remove { paths: vec![] }),
        "Rename" => Ok(FopsAction::Rename { src: None, dest: None }),
        "Chmod" => Ok(FopsAction::Chmod { paths: vec![] }),
        "Size" => Ok(FopsAction::Size { paths: vec![] }),
        "Netinfo" => Ok(FopsAction::Netinfo),
        _ => anyhow::bail!("Invalid action selected"),
    }
}

fn get_current_dir_entries() -> Result<Vec<String>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        let name = path.strip_prefix(".").unwrap_or(&path).to_string_lossy().into_owned();
        if name != ".git" { // 隐藏底层的 .git，避免误操作
            entries.push(name);
        }
    }
    entries.sort();
    Ok(entries)
}

fn prompt_multi_files(prompt: &str) -> Result<Vec<String>> {
    let files = get_current_dir_entries()?;
    if files.is_empty() {
        anyhow::bail!("No files found in the current directory.");
    }
    let selected = MultiSelect::new(prompt, files)
        .with_render_config(get_theme())
        .prompt()?;
    if selected.is_empty() {
        anyhow::bail!("No files selected.");
    }
    Ok(selected)
}

fn prompt_single_file(prompt: &str) -> Result<String> {
    let files = get_current_dir_entries()?;
    if files.is_empty() {
        anyhow::bail!("No files found in the current directory.");
    }
    let selected = Select::new(prompt, files)
        .with_render_config(get_theme())
        .prompt()?;
    Ok(selected)
}

fn size_path(path: &str) -> Result<()> {
    let metadata = fs::metadata(path)?;
    let size = if metadata.is_dir() {
        get_dir_size(path)?
    } else {
        metadata.len()
    };
    
    let formatted_size = format_size(size);
    println!("{}: {}", path.cyan(), formatted_size.yellow());
    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

fn get_dir_size(path: impl AsRef<Path>) -> Result<u64> {
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            size += get_dir_size(entry.path())?;
        } else {
            size += metadata.len();
        }
    }
    Ok(size)
}

fn copy_paths(srcs: &[String], dest: &str) -> Result<()> {
    let dest_path = Path::new(dest);
    let is_multi = srcs.len() > 1;

    if is_multi && !dest_path.exists() {
        fs::create_dir_all(dest_path)?;
    }

    for src in srcs {
        let src_path = Path::new(src);
        if !src_path.exists() {
            println!("{}", format!("Source {} does not exist, skipping.", src).red());
            continue;
        }

        let target = if is_multi || dest_path.is_dir() {
            let file_name = src_path.file_name().unwrap();
            dest_path.join(file_name)
        } else {
            dest_path.to_path_buf()
        };

        let metadata = fs::metadata(&src_path)?;
        if metadata.is_dir() {
            copy_dir(&src_path, &target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src_path, &target)?;
        }
        println!("{}", format!("Successfully copied {} to {}", src, target.display()).green());
    }
    Ok(())
}

fn copy_dir(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir(entry.path(), dest.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dest.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn move_paths(srcs: &[String], dest: &str) -> Result<()> {
    let dest_path = Path::new(dest);
    let is_multi = srcs.len() > 1;

    if is_multi && !dest_path.exists() {
        fs::create_dir_all(dest_path)?;
    }

    for src in srcs {
        let src_path = Path::new(src);
        if !src_path.exists() {
            println!("{}", format!("Source {} does not exist, skipping.", src).red());
            continue;
        }

        let target = if is_multi || dest_path.is_dir() {
            let file_name = src_path.file_name().unwrap();
            dest_path.join(file_name)
        } else {
            dest_path.to_path_buf()
        };

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&src_path, &target)?;
        println!("{}", format!("Successfully moved {} to {}", src, target.display()).green());
    }
    Ok(())
}

fn rename_path(src: &str, dest: &str) -> Result<()> {
    fs::rename(src, dest)?;
    println!("{}", format!("Successfully renamed {} to {}", src, dest).green());
    Ok(())
}

fn remove_path(path: &str) -> Result<()> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    println!("{}", format!("Successfully removed {}", path).green());
    Ok(())
}

fn chmod_interactive(paths: Vec<String>) -> Result<()> {
    if paths.is_empty() { return Ok(()); }
    
    // 以第一个文件的权限作为勾选的默认值展示
    let first_path = Path::new(&paths[0]);
    let metadata = fs::metadata(&first_path)?;
    let perms = metadata.permissions();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = perms.mode();
        let options = vec![
            "Owner Read", "Owner Write", "Owner Execute",
            "Group Read", "Group Write", "Group Execute",
            "Other Read", "Other Write", "Other Execute"
        ];
        let mut defaults = Vec::new();
        if mode & 0o400 != 0 { defaults.push(0); }
        if mode & 0o200 != 0 { defaults.push(1); }
        if mode & 0o100 != 0 { defaults.push(2); }
        if mode & 0o040 != 0 { defaults.push(3); }
        if mode & 0o020 != 0 { defaults.push(4); }
        if mode & 0o010 != 0 { defaults.push(5); }
        if mode & 0o004 != 0 { defaults.push(6); }
        if mode & 0o002 != 0 { defaults.push(7); }
        if mode & 0o001 != 0 { defaults.push(8); }

        let prompt_msg = if paths.len() == 1 {
            format!("Select permissions for '{}':", paths[0])
        } else {
            format!("Select permissions for {} files/dirs (Batch apply):", paths.len())
        };

        let selection = MultiSelect::new(&prompt_msg, options)
            .with_default(&defaults)
            .with_render_config(get_theme())
            .prompt()?;

        let mut new_mode = mode & !0o777; 
        for sel in selection {
            match sel {
                "Owner Read" => new_mode |= 0o400,
                "Owner Write" => new_mode |= 0o200,
                "Owner Execute" => new_mode |= 0o100,
                "Group Read" => new_mode |= 0o040,
                "Group Write" => new_mode |= 0o020,
                "Group Execute" => new_mode |= 0o010,
                "Other Read" => new_mode |= 0o004,
                "Other Write" => new_mode |= 0o002,
                "Other Execute" => new_mode |= 0o001,
                _ => {}
            }
        }
        
        for p in paths {
            let mut p_perms = fs::metadata(&p)?.permissions();
            p_perms.set_mode(new_mode);
            fs::set_permissions(&p, p_perms)?;
            println!("{}", format!("Permissions updated for {}", p).green());
        }
    }

    #[cfg(not(unix))]
    {
        let options = vec!["Read-only"];
        let mut defaults = Vec::new();
        if perms.readonly() {
            defaults.push(0);
        }

        let prompt_msg = if paths.len() == 1 {
            format!("Select permissions for '{}':", paths[0])
        } else {
            format!("Select permissions for {} files/dirs (Batch apply):", paths.len())
        };

        let selection = MultiSelect::new(&prompt_msg, options)
            .with_default(&defaults)
            .with_render_config(get_theme())
            .prompt()?;

        let set_readonly = selection.contains(&"Read-only");

        for p in paths {
            let mut p_perms = fs::metadata(&p)?.permissions();
            p_perms.set_readonly(set_readonly);
            fs::set_permissions(&p, p_perms)?;
            println!("{}", format!("Permissions updated for {}", p).green());
        }
    }

    Ok(())
}

fn netinfo() -> Result<()> {
    use sysinfo::Networks;
    let networks = Networks::new_with_refreshed_list();
    
    let (default_gateway, dns_servers) = get_gateway_and_dns();
    let mut printed_gateway = false;

    println!();
    for (interface_name, data) in &networks {
        let mac = data.mac_address();
        let ips = data.ip_networks();
        
        let mut has_ipv4 = false;
        let mut ipv4_str = String::new();
        for ip in ips {
            let addr = ip.addr;
            if addr.is_ipv4() {
                ipv4_str = format!("{}/{}", addr, ip.prefix);
                has_ipv4 = true;
                break;
            }
        }

        let is_up = !ips.is_empty();
        let status_str = if is_up { "Running" } else { "Disconnected" };

        println!("Interface Name: {}  [ {} ]", interface_name.cyan(), status_str.yellow());
        println!("  Physical Address (MAC): {}", mac);
        
        if has_ipv4 {
            println!("  IPv4 Address/Mask: {}", ipv4_str);
            if !printed_gateway {
                if let Some(ref gw) = default_gateway {
                    println!("  Default Gateway: {} (Main Internet Interface)", gw);
                    printed_gateway = true;
                }
            }
        } else {
            println!("  IPv4 Address: Not acquired");
        }
        println!("----------------------------------------------------------------");
    }
    
    println!("DNS Servers:");
    if dns_servers.is_empty() {
        println!("  - Unknown");
    } else {
        for dns in dns_servers {
            println!("  - {}", dns);
        }
    }
    
    Ok(())
}

fn get_gateway_and_dns() -> (Option<String>, Vec<String>) {
    let mut gateway = None;
    let mut dns_servers = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("ipconfig").arg("/all").output() {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut in_dns = false;
            for line in text.lines() {
                let line_trim = line.trim();
                if line_trim.contains(":") {
                    let parts: Vec<&str> = line_trim.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let key = parts[0].trim();
                        let val = parts[1].trim();
                        if (key.contains("Default Gateway") || key.contains("默认网关") || key.contains("Gateway")) && !val.is_empty() {
                            if gateway.is_none() && !val.contains("::") && !val.contains(":") { 
                                gateway = Some(val.to_string());
                            } else if gateway.is_none() && val.matches('.').count() == 3 {
                                gateway = Some(val.to_string());
                            }
                        } else if key.contains("DNS Servers") || key.contains("DNS 服务器") || key.contains("DNS") {
                            if !val.is_empty() && !val.contains("fec0:") && !val.contains("::") {
                                dns_servers.push(val.to_string());
                                in_dns = true;
                            }
                        } else {
                            in_dns = false;
                        }
                    }
                } else if in_dns && !line_trim.is_empty() {
                    if !line_trim.contains("fec0:") && !line_trim.contains("::") && line_trim.matches('.').count() == 3 {
                        dns_servers.push(line_trim.to_string());
                    }
                } else {
                    in_dns = false;
                }
            }
        }
    }

    #[cfg(unix)]
    {
        if cfg!(target_os = "macos") {
            if let Ok(output) = std::process::Command::new("route").arg("-n").arg("get").arg("default").output() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    let line_trim = line.trim();
                    if line_trim.starts_with("gateway:") {
                        let parts: Vec<&str> = line_trim.split_whitespace().collect();
                        if parts.len() >= 2 {
                            gateway = Some(parts[1].to_string());
                            break;
                        }
                    }
                }
            }
        } else {
            if let Ok(output) = std::process::Command::new("ip").arg("route").output() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.starts_with("default via") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            gateway = Some(parts[2].to_string());
                            break;
                        }
                    }
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                let line_trim = line.trim();
                if line_trim.starts_with("nameserver") {
                    let parts: Vec<&str> = line_trim.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let ip = parts[1].to_string();
                        if !dns_servers.contains(&ip) {
                            dns_servers.push(ip);
                        }
                    }
                }
            }
        } 
        
        if dns_servers.is_empty() {
            if let Ok(output) = std::process::Command::new("resolvectl").arg("status").output() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if line.contains("DNS Servers:") {
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            for ip in parts[1].split_whitespace() {
                                if !dns_servers.contains(&ip.to_string()) {
                                    dns_servers.push(ip.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (gateway, dns_servers)
}