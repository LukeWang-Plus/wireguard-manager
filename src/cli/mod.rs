//! CLI argument definitions (clap derive) and command dispatch.

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

use crate::manager::{self, DeviceInfo, MessageKind, OperationResult, WireGuardManager};

#[derive(Parser)]
#[command(
    name = "wireguard",
    about = "WireGuard Network Configuration Management Tool"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize network configuration
    Init {
        /// Subnet in CIDR format (e.g., 10.0.0.0/24)
        #[arg(long)]
        subnet: String,
        /// Server IP offset range (e.g., [1,10])
        #[arg(long = "server-range")]
        server_range: String,
        /// Force reinitialize (WARNING: deletes existing data)
        #[arg(long)]
        force: bool,
    },
    /// View or modify network configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Add device
    Add {
        #[command(subcommand)]
        device_type: AddDevice,
    },
    /// Edit device
    Edit {
        #[command(subcommand)]
        device_type: EditDevice,
    },
    /// Delete device
    Del {
        #[command(subcommand)]
        device_type: DelDevice,
    },
    /// Generate config
    Gen {
        /// Device name or 'all'
        name: String,
        /// Output directory
        #[arg(long = "output-dir")]
        output_dir: Option<String>,
    },
    /// List all devices
    List,
    /// Show device details
    Show {
        /// Device name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current network configuration
    Show,
    /// Modify network configuration
    Set {
        /// New subnet in CIDR format
        #[arg(long)]
        subnet: Option<String>,
        /// New server IP offset range (e.g., [1,20])
        #[arg(long = "server-range")]
        server_range: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AddDevice {
    /// Add server
    Server {
        /// Server name
        name: String,
        /// Public IP
        #[arg(long = "public-ip")]
        public_ip: String,
        /// Listen port (default 51820)
        #[arg(long, default_value_t = 51820)]
        port: u16,
    },
    /// Add client
    Client {
        /// Client name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum EditDevice {
    /// Edit server
    Server {
        /// Server name
        name: String,
        /// New public IP
        #[arg(long = "public-ip")]
        public_ip: Option<String>,
        /// New listen port
        #[arg(long)]
        port: Option<u16>,
    },
    /// Edit client
    Client {
        /// Client name
        name: String,
        /// New name
        #[arg(long = "new-name")]
        new_name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DelDevice {
    /// Delete server
    Server {
        /// Server name
        name: String,
    },
    /// Delete client
    Client {
        /// Client name
        name: String,
    },
}

fn print_messages(result: &OperationResult) {
    for msg in &result.messages {
        match msg.kind {
            MessageKind::Success | MessageKind::Warning => println!("{}", msg.text),
            MessageKind::Info => println!("  {}", msg.text),
            MessageKind::Error => eprintln!("Error: {}", msg.text),
        }
    }
}

/// Execute the given CLI command against a default `WireGuardManager`.
pub fn run(cmd: Commands) {
    let mut mgr = WireGuardManager::new("data/data.json");

    let result: Result<()> = match cmd {
        Commands::Init {
            subnet,
            server_range,
            force,
        } => handle_init(&mut mgr, &subnet, &server_range, force),
        Commands::Config { action } => handle_config(&mut mgr, action),
        Commands::Add { device_type } => handle_add(&mut mgr, device_type),
        Commands::Edit { device_type } => handle_edit(&mut mgr, device_type),
        Commands::Del { device_type } => handle_del(&mut mgr, device_type),
        Commands::Gen { name, output_dir } => handle_gen(&mgr, &name, output_dir.as_deref()),
        Commands::List => handle_list(&mgr),
        Commands::Show { name } => handle_show(&mgr, &name),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}

fn handle_init(
    mgr: &mut WireGuardManager,
    subnet: &str,
    server_range: &str,
    force: bool,
) -> Result<()> {
    let Ok((start, end)) = manager::parse_server_range(server_range) else {
        bail!("Invalid server range '{server_range}'. Expected format: [START,END] (e.g., [1,10])");
    };
    let result = mgr.init_network(subnet, start, end, force)?;
    print_messages(&result);
    Ok(())
}

fn handle_config(mgr: &mut WireGuardManager, action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let info = mgr.show_network_config()?;
            println!("\n=== Network Configuration ===");
            println!("  Subnet: {}", info.subnet);
            println!(
                "  Server IP range: {} - {} (offset {}-{})",
                info.server_start_ip,
                info.server_end_ip,
                info.server_range_offsets.0,
                info.server_range_offsets.1
            );
            println!(
                "  Client IP range: {} - {} (offset {}-{})",
                info.client_start_ip,
                info.client_end_ip,
                info.client_start_offset,
                info.max_host_offset
            );
            println!("\n=== Usage Statistics ===");
            println!(
                "  Servers: {}/{} ({} available)",
                info.server_count,
                info.server_capacity,
                info.server_capacity.saturating_sub(info.server_count)
            );
            println!(
                "  Clients: {}/{} ({} available)",
                info.client_count,
                info.client_capacity,
                info.client_capacity.saturating_sub(info.client_count)
            );
            println!();
            Ok(())
        }
        ConfigAction::Set {
            subnet,
            server_range,
        } => {
            let (ss, se) = if let Some(ref sr) = server_range {
                let Ok((s, e)) = manager::parse_server_range(sr) else {
                    bail!(
                        "Invalid server range '{sr}'. Expected format: [START,END] (e.g., [1,20])"
                    );
                };
                (Some(s), Some(e))
            } else {
                (None, None)
            };
            let result = mgr.update_network_config(subnet.as_deref(), ss, se)?;
            print_messages(&result);
            Ok(())
        }
    }
}

fn handle_add(mgr: &mut WireGuardManager, device_type: AddDevice) -> Result<()> {
    let result = match device_type {
        AddDevice::Server {
            name,
            public_ip,
            port,
        } => mgr.add_server(&name, &public_ip, port)?,
        AddDevice::Client { name } => mgr.add_client(&name)?,
    };
    print_messages(&result);
    Ok(())
}

fn handle_edit(mgr: &mut WireGuardManager, device_type: EditDevice) -> Result<()> {
    let result = match device_type {
        EditDevice::Server {
            name,
            public_ip,
            port,
        } => mgr.edit_server(&name, public_ip.as_deref(), port)?,
        EditDevice::Client { name, new_name } => mgr.edit_client(&name, new_name.as_deref())?,
    };
    print_messages(&result);
    Ok(())
}

fn handle_del(mgr: &mut WireGuardManager, device_type: DelDevice) -> Result<()> {
    let result = match device_type {
        DelDevice::Server { name } => mgr.delete_server(&name)?,
        DelDevice::Client { name } => mgr.delete_client(&name)?,
    };
    print_messages(&result);
    Ok(())
}

fn handle_gen(mgr: &WireGuardManager, name: &str, output_dir: Option<&str>) -> Result<()> {
    if name == "all" {
        let dir = output_dir.unwrap_or("output");
        let count = mgr.generate_all_configs(dir)?;
        println!("Generated {count} config files to: {dir}");
    } else if let Some(config) = mgr.generate_config(name, output_dir)? {
        println!("\n=== {name} Config ===");
        println!("{config}");
        println!();
    } else {
        let dir = output_dir.unwrap_or("output");
        println!("Config saved to: {dir}/{name}.conf");
    }
    Ok(())
}

fn handle_list(mgr: &WireGuardManager) -> Result<()> {
    let (servers, clients) = mgr.list_devices()?;
    println!("\n=== Servers ===");
    if servers.is_empty() {
        println!("  (none)");
    } else {
        for server in servers.values() {
            println!(
                "  {}: {} (Public: {}:{})",
                server.name, server.ip, server.public_ip, server.listen_port
            );
        }
    }
    println!("\n=== Clients ===");
    if clients.is_empty() {
        println!("  (none)");
    } else {
        for client in clients.values() {
            println!("  {}: {}", client.name, client.ip);
        }
    }
    println!();
    Ok(())
}

fn handle_show(mgr: &WireGuardManager, name: &str) -> Result<()> {
    match mgr.show_device(name)? {
        DeviceInfo::Server {
            name,
            ip,
            public_ip,
            listen_port,
            private_key,
            public_key,
            preshared_keys_count,
        } => {
            println!("\n=== Server: {name} ===");
            println!("  Internal IP: {ip}");
            println!("  Public IP: {public_ip}");
            println!("  Port: {listen_port}");
            println!("  Private Key: {private_key}");
            println!("  Public Key: {public_key}");
            println!("  Preshared Keys Count: {preshared_keys_count}");
            println!();
        }
        DeviceInfo::Client {
            name,
            ip,
            private_key,
            public_key,
        } => {
            println!("\n=== Client: {name} ===");
            println!("  Internal IP: {ip}");
            println!("  Private Key: {private_key}");
            println!("  Public Key: {public_key}");
            println!();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
