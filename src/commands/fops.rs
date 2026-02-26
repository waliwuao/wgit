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
        FopsAction::Copy { src, dest } => copy_path(&src, &dest)?,
        FopsAction::Move { src, dest } => move_path(&src, &dest)?,
        FopsAction::Remove { path } => remove_path(&path)?,
        FopsAction::Rename { src, dest } => rename_path(&src, &dest)?,
        FopsAction::Chmod { path } => chmod_interactive(path)?,
        FopsAction::Size { path } => size_path(&path)?,
        FopsAction::Netinfo => netinfo()?,
    }
    Ok(())
}

fn interactive_select() -> Result<FopsAction> {
    let actions = vec![
        format!("{:<14} {}", "Copy", "Copy file or directory".bright_black()),
        format!("{:<14} {}", "Move", "Move file or directory".bright_black()),
        format!("{:<14} {}", "Remove", "Delete file or directory".bright_black()),
        format!("{:<14} {}", "Rename", "Rename file or directory".bright_black()),
        format!("{:<14} {}", "Chmod", "Change file permissions".bright_black()),
        format!("{:<14} {}", "Size", "Get size of file or directory".bright_black()),
        format!("{:<14} {}", "Netinfo", "Print network interfaces info".bright_black()),
    ];

    let choice = Select::new("Select fops action:", actions)
        .with_render_config(get_theme())
        .with_page_size(10)
        .prompt()?;

    let cmd_str = choice.split_whitespace().next().unwrap_or("");
    match cmd_str {
        "Copy" => {
            let src = Text::new("Source path:").with_render_config(get_theme()).prompt()?;
            let dest = Text::new("Destination path:").with_render_config(get_theme()).prompt()?;
            Ok(FopsAction::Copy { src, dest })
        }
        "Move" => {
            let src = Text::new("Source path:").with_render_config(get_theme()).prompt()?;
            let dest = Text::new("Destination path:").with_render_config(get_theme()).prompt()?;
            Ok(FopsAction::Move { src, dest })
        }
        "Remove" => {
            let path = Text::new("Target path:").with_render_config(get_theme()).prompt()?;
            Ok(FopsAction::Remove { path })
        }
        "Rename" => {
            let src = Text::new("Source path:").with_render_config(get_theme()).prompt()?;
            let dest = Text::new("New name/path:").with_render_config(get_theme()).prompt()?;
            Ok(FopsAction::Rename { src, dest })
        }
        "Chmod" => {
            let path = Text::new("Target path:").with_render_config(get_theme()).prompt()?;
            Ok(FopsAction::Chmod { path: Some(path) })
        }
        "Size" => {
            let path = Text::new("Target path:").with_render_config(get_theme()).prompt()?;
            Ok(FopsAction::Size { path })
        }
        "Netinfo" => Ok(FopsAction::Netinfo),
        _ => anyhow::bail!("Invalid action selected"),
    }
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

fn copy_path(src: &str, dest: &str) -> Result<()> {
    let metadata = fs::metadata(src)?;
    if metadata.is_dir() {
        copy_dir(src, dest)?;
    } else {
        fs::copy(src, dest)?;
    }
    println!("{}", format!("Successfully copied {} to {}", src, dest).green());
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

fn move_path(src: &str, dest: &str) -> Result<()> {
    fs::rename(src, dest)?;
    println!("{}", format!("Successfully moved {} to {}", src, dest).green());
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

fn chmod_interactive(path_arg: Option<String>) -> Result<()> {
    let path_str = match path_arg {
        Some(p) => p,
        None => {
            Text::new("Enter file/dir path to change permissions:")
                .with_render_config(get_theme())
                .prompt()?
        }
    };

    let path = Path::new(&path_str);
    let metadata = fs::metadata(&path)?;
    let mut perms = metadata.permissions();

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

        let selection = MultiSelect::new("Select permissions:", options.clone())
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
        perms.set_mode(new_mode);
        fs::set_permissions(&path, perms)?;
        println!("{}", format!("Permissions updated for {}", path_str).green());
    }

    #[cfg(not(unix))]
    {
        let options = vec!["Read-only"];
        let mut defaults = Vec::new();
        if perms.readonly() {
            defaults.push(0);
        }

        let selection = MultiSelect::new("Select permissions:", options.clone())
            .with_default(&defaults)
            .with_render_config(get_theme())
            .prompt()?;

        if selection.contains(&"Read-only") {
            perms.set_readonly(true);
        } else {
            perms.set_readonly(false);
        }
        fs::set_permissions(&path, perms)?;
        println!("{}", format!("Permissions updated for {}", path_str).green());
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