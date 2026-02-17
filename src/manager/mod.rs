//! Core business logic: network initialization, device management, IP allocation, and config generation.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Local;
use ipnet::Ipv4Net;
use serde::{Deserialize, Serialize};

use crate::key::{generate_keypair, generate_preshared_key};

// ── Data structures ──────────────────────────────────────────────────

/// Persisted network configuration: subnet and IP allocation ranges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub subnet: String,
    /// Server IP offset range within the subnet `[start, end]`.
    pub server_ip_range: [u32; 2],
    /// Host offset at which client IP allocation begins.
    pub client_ip_start: u32,
}

/// A server device with its keys, endpoint info, and per-peer pre-shared keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub ip: String,
    pub public_ip: String,
    pub listen_port: u16,
    pub private_key: String,
    pub public_key: String,
    /// Pre-shared keys indexed by peer device name.
    pub preshared_keys: BTreeMap<String, String>,
}

/// A client device with its keys and assigned IP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientEntry {
    pub name: String,
    pub ip: String,
    pub private_key: String,
    pub public_key: String,
}

/// Top-level JSON data structure persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFile {
    pub network: NetworkConfig,
    pub servers: BTreeMap<String, ServerEntry>,
    pub clients: BTreeMap<String, ClientEntry>,
}

// ── Result types for UI layer ────────────────────────────────────────

/// Computed network statistics for UI display.
pub struct NetworkInfo {
    pub subnet: String,
    pub server_start_ip: Ipv4Addr,
    pub server_end_ip: Ipv4Addr,
    pub client_start_ip: Ipv4Addr,
    pub client_end_ip: Ipv4Addr,
    pub server_range_offsets: (u32, u32),
    pub client_start_offset: u32,
    pub max_host_offset: u32,
    pub server_count: u32,
    pub server_capacity: u32,
    pub client_count: u32,
    pub client_capacity: u32,
}

/// Severity level for operation feedback messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageKind {
    Success,
    Info,
    Warning,
    #[allow(dead_code)]
    Error,
}

/// A single operation feedback message with a severity level.
#[derive(Debug, Clone)]
pub struct Message {
    pub kind: MessageKind,
    pub text: String,
}

impl Message {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Success,
            text: text.into(),
        }
    }
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Info,
            text: text.into(),
        }
    }
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            kind: MessageKind::Warning,
            text: text.into(),
        }
    }
}

/// Result of a manager operation, carrying feedback messages and an optional backup path.
#[derive(Debug)]
pub struct OperationResult {
    pub messages: Vec<Message>,
    /// Backup file path if created. Reserved for future TUI use.
    #[allow(dead_code)]
    pub backup_path: Option<PathBuf>,
}

impl OperationResult {
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            messages: vec![Message::success(text)],
            backup_path: None,
        }
    }

    pub fn success_text(&self) -> Option<&str> {
        self.messages
            .iter()
            .find(|m| m.kind == MessageKind::Success)
            .map(|m| m.text.as_str())
    }
}

/// Type alias for the (servers, clients) tuple returned by `list_devices`.
pub type DeviceList<'a> = (
    &'a BTreeMap<String, ServerEntry>,
    &'a BTreeMap<String, ClientEntry>,
);

// ── Manager ──────────────────────────────────────────────────────────

/// Core manager: owns the data file path, caches parsed network state, and
/// exposes the public API for all `WireGuard` management operations.
pub struct WireGuardManager {
    data_file: PathBuf,
    network: Option<Ipv4Net>,
    server_ip_range: Option<(u32, u32)>,
    client_ip_start: Option<u32>,
    data: Option<DataFile>,
}

impl WireGuardManager {
    /// Create a new manager backed by the given JSON data file path.
    ///
    /// If the file already exists it is loaded immediately.
    pub fn new(data_file: &str) -> Self {
        let mut mgr = Self {
            data_file: PathBuf::from(data_file),
            network: None,
            server_ip_range: None,
            client_ip_start: None,
            data: None,
        };
        let _ = mgr.load_data();
        mgr
    }

    // ── Helpers ──────────────────────────────────────────────────────

    fn network_int(&self) -> Result<u32> {
        Ok(u32::from(
            self.network
                .ok_or_else(|| anyhow!("Network not initialized"))?
                .network(),
        ))
    }

    fn max_host_offset(&self) -> Result<u32> {
        let net = self
            .network
            .ok_or_else(|| anyhow!("Network not initialized"))?;
        let num_addresses = 1u32 << (32 - net.prefix_len());
        Ok(num_addresses - 2)
    }

    fn ip_from_offset(&self, offset: u32) -> Result<Ipv4Addr> {
        Ok(Ipv4Addr::from(self.network_int()? + offset))
    }

    fn data(&self) -> Result<&DataFile> {
        self.data
            .as_ref()
            .ok_or_else(|| anyhow!("No configuration loaded"))
    }

    fn data_mut(&mut self) -> Result<&mut DataFile> {
        self.data
            .as_mut()
            .ok_or_else(|| anyhow!("No configuration loaded"))
    }

    fn ensure_loaded(&self) -> Result<()> {
        if self.data.is_none() {
            bail!("No configuration found. Please run 'init' first.");
        }
        Ok(())
    }

    // ── Persistence ──────────────────────────────────────────────────

    fn load_data(&mut self) -> Result<()> {
        if !self.data_file.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&self.data_file)
            .with_context(|| format!("Failed to read {}", self.data_file.display()))?;
        let data: DataFile = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", self.data_file.display()))?;

        self.network = Some(
            data.network
                .subnet
                .parse::<Ipv4Net>()
                .context("Invalid subnet in data file")?,
        );
        self.server_ip_range = Some((
            data.network.server_ip_range[0],
            data.network.server_ip_range[1],
        ));
        self.client_ip_start = Some(data.network.client_ip_start);
        self.data = Some(data);
        Ok(())
    }

    fn save_data(&self) -> Result<()> {
        if let Some(parent) = self.data_file.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let content =
            serde_json::to_string_pretty(self.data()?).context("Failed to serialize data")?;
        fs::write(&self.data_file, content)
            .with_context(|| format!("Failed to write {}", self.data_file.display()))?;
        Ok(())
    }

    fn backup_data(&self) -> Result<PathBuf> {
        let date_str = Local::now().format("%Y%m%d%H%M%S").to_string();
        let backup_name = format!("data-{date_str}.json.bak");
        let backup_path = self.data_file.with_file_name(backup_name);
        fs::copy(&self.data_file, &backup_path).with_context(|| "Failed to create backup")?;
        Ok(backup_path)
    }

    // ── IP allocation ────────────────────────────────────────────────

    fn get_next_ip(
        &self,
        start: u32,
        end: u32,
        used_ips: &HashSet<&str>,
    ) -> Result<Option<String>> {
        for i in start..=end {
            let ip = self.ip_from_offset(i)?.to_string();
            if !used_ips.contains(ip.as_str()) {
                return Ok(Some(ip));
            }
        }
        Ok(None)
    }

    fn get_next_server_ip(&self) -> Result<Option<String>> {
        let range = self
            .server_ip_range
            .ok_or_else(|| anyhow!("Server IP range not initialized"))?;
        let used: HashSet<&str> = self
            .data()?
            .servers
            .values()
            .map(|s| s.ip.as_str())
            .collect();
        self.get_next_ip(range.0, range.1, &used)
    }

    fn get_next_client_ip(&self) -> Result<Option<String>> {
        let start = self
            .client_ip_start
            .ok_or_else(|| anyhow!("Client IP start not initialized"))?;
        let used: HashSet<&str> = self
            .data()?
            .clients
            .values()
            .map(|c| c.ip.as_str())
            .collect();
        self.get_next_ip(start, self.max_host_offset()?, &used)
    }

    fn name_exists(&self, name: &str) -> Result<bool> {
        let data = self.data()?;
        Ok(data.servers.contains_key(name) || data.clients.contains_key(name))
    }

    fn get_device_type(&self, name: &str) -> Result<Option<&str>> {
        let data = self.data()?;
        if data.servers.contains_key(name) {
            Ok(Some("server"))
        } else if data.clients.contains_key(name) {
            Ok(Some("client"))
        } else {
            Ok(None)
        }
    }

    fn save_config_file(name: &str, config: &str, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create directory {}", output_dir.display()))?;
        let path = output_dir.join(format!("{name}.conf"));
        fs::write(&path, config).with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    /// Ensure no existing server or client IP falls outside the proposed new range.
    fn validate_server_range_change(&self, new_start: u32, new_end: u32) -> Result<()> {
        if new_start < 1 {
            bail!("Server start offset must be >= 1, got {new_start}");
        }
        if new_end > self.max_host_offset()? {
            bail!(
                "Server end offset must be <= {} for this network, got {new_end}",
                self.max_host_offset()?
            );
        }
        if new_start > new_end {
            bail!("Server start ({new_start}) must be <= server end ({new_end})");
        }

        let new_start_ip = self.ip_from_offset(new_start)?;
        let new_end_ip = self.ip_from_offset(new_end)?;

        let out_of_range: Vec<String> = self
            .data()?
            .servers
            .iter()
            .filter_map(|(name, server)| {
                let ip: Ipv4Addr = server.ip.parse().ok()?;
                if ip < new_start_ip || ip > new_end_ip {
                    Some(format!("{name} ({ip})"))
                } else {
                    None
                }
            })
            .collect();

        if !out_of_range.is_empty() {
            bail!(
                "Cannot change server range: the following servers would be out of range:\n  {}\nPlease delete these servers first or choose a range that includes them.",
                out_of_range.join(", ")
            );
        }

        let new_client_start_ip = self.ip_from_offset(new_end + 1)?;
        let conflicting: Vec<String> = self
            .data()?
            .clients
            .iter()
            .filter_map(|(name, client)| {
                let ip: Ipv4Addr = client.ip.parse().ok()?;
                if ip < new_client_start_ip {
                    Some(format!("{name} ({ip})"))
                } else {
                    None
                }
            })
            .collect();

        if !conflicting.is_empty() {
            bail!(
                "Cannot change server range: the following clients would conflict with server range:\n  {}\nPlease delete these clients first or choose a smaller server range.",
                conflicting.join(", ")
            );
        }

        Ok(())
    }

    // ── Public API ───────────────────────────────────────────────────

    /// Initialize (or force-reinitialize) the `WireGuard` network.
    ///
    /// Creates the data file with the given subnet and server IP offset range.
    /// If `force` is true and the file exists, a timestamped backup is created first.
    pub fn init_network(
        &mut self,
        subnet: &str,
        server_start: u32,
        server_end: u32,
        force: bool,
    ) -> Result<OperationResult> {
        let mut messages = Vec::new();

        if self.data_file.exists() && !force {
            bail!(
                "Data file '{}' already exists.\nUse --force to reinitialize (WARNING: This will delete all existing data!)",
                self.data_file.display()
            );
        }

        let network: Ipv4Net = subnet
            .parse::<Ipv4Net>()
            .with_context(|| format!("Invalid subnet '{subnet}'"))?
            .trunc();

        let num_addresses = 1u32 << (32 - network.prefix_len());
        let max_host_offset = num_addresses.saturating_sub(2);

        if max_host_offset < 1 {
            bail!("Network '{subnet}' is too small (no usable host addresses)");
        }
        if server_start < 1 {
            bail!("Server start offset must be >= 1, got {server_start}");
        }
        if server_end > max_host_offset {
            bail!(
                "Server end offset must be <= {max_host_offset} for this network, got {server_end}"
            );
        }
        if server_start > server_end {
            bail!("Server start ({server_start}) must be <= server end ({server_end})");
        }

        let client_ip_start = server_end + 1;
        if client_ip_start > max_host_offset {
            bail!("No IP addresses left for clients (server range ends at offset {server_end})");
        }

        let backup_path = if self.data_file.exists() && force {
            let backup = self.backup_data()?;
            messages.push(Message::warning(format!(
                "Old data backed up to: {}",
                backup.display()
            )));
            Some(backup)
        } else {
            None
        };

        self.network = Some(network);
        self.server_ip_range = Some((server_start, server_end));
        self.client_ip_start = Some(client_ip_start);

        let net_int = u32::from(network.network());
        let server_start_ip = Ipv4Addr::from(net_int + server_start);
        let server_end_ip = Ipv4Addr::from(net_int + server_end);
        let client_start_ip = Ipv4Addr::from(net_int + client_ip_start);
        let client_end_ip = Ipv4Addr::from(net_int + max_host_offset);

        self.data = Some(DataFile {
            network: NetworkConfig {
                subnet: network.to_string(),
                server_ip_range: [server_start, server_end],
                client_ip_start,
            },
            servers: BTreeMap::new(),
            clients: BTreeMap::new(),
        });

        self.save_data()?;

        messages.push(Message::success("Network initialized successfully!"));
        messages.push(Message::info(format!("Subnet: {network}")));
        messages.push(Message::info(format!(
            "Server IP range: {server_start_ip} - {server_end_ip}"
        )));
        messages.push(Message::info(format!(
            "Client IP range: {client_start_ip} - {client_end_ip}"
        )));

        Ok(OperationResult {
            messages,
            backup_path,
        })
    }

    /// Return computed network statistics for the current configuration.
    pub fn show_network_config(&self) -> Result<NetworkInfo> {
        self.ensure_loaded()?;

        let data = self.data()?;
        let net_cfg = &data.network;
        let range = self
            .server_ip_range
            .ok_or_else(|| anyhow!("Server IP range not initialized"))?;
        let client_start = self
            .client_ip_start
            .ok_or_else(|| anyhow!("Client IP start not initialized"))?;
        let max_offset = self.max_host_offset()?;

        let server_count: u32 = data
            .servers
            .len()
            .try_into()
            .context("Server count exceeds u32 range")?;
        let client_count: u32 = data
            .clients
            .len()
            .try_into()
            .context("Client count exceeds u32 range")?;

        Ok(NetworkInfo {
            subnet: net_cfg.subnet.clone(),
            server_start_ip: self.ip_from_offset(range.0)?,
            server_end_ip: self.ip_from_offset(range.1)?,
            client_start_ip: self.ip_from_offset(client_start)?,
            client_end_ip: self.ip_from_offset(max_offset)?,
            server_range_offsets: range,
            client_start_offset: client_start,
            max_host_offset: max_offset,
            server_count,
            server_capacity: range.1 - range.0 + 1,
            client_count,
            client_capacity: max_offset - client_start + 1,
        })
    }

    /// Update subnet and/or server IP range, with validation against existing devices.
    pub fn update_network_config(
        &mut self,
        subnet: Option<&str>,
        server_start: Option<u32>,
        server_end: Option<u32>,
    ) -> Result<OperationResult> {
        self.ensure_loaded()?;
        let mut messages = Vec::new();
        let mut has_changes = false;

        if let Some(subnet_str) = subnet {
            let new_network: Ipv4Net = subnet_str
                .parse::<Ipv4Net>()
                .with_context(|| format!("Invalid subnet '{subnet_str}'"))?
                .trunc();

            if !self.data()?.servers.is_empty() || !self.data()?.clients.is_empty() {
                bail!(
                    "Cannot change subnet when devices exist.\nPlease delete all servers and clients first, or use 'init --force' to reinitialize."
                );
            }

            self.network = Some(new_network);
            self.data_mut()?.network.subnet = new_network.to_string();
            has_changes = true;
            messages.push(Message::info(format!("Subnet updated to: {new_network}")));
        }

        if server_start.is_some() || server_end.is_some() {
            let old_range = self
                .server_ip_range
                .ok_or_else(|| anyhow!("Server IP range not initialized"))?;
            let new_start = server_start.unwrap_or(old_range.0);
            let new_end = server_end.unwrap_or(old_range.1);

            self.validate_server_range_change(new_start, new_end)?;

            let new_client_start = new_end + 1;
            if new_client_start > self.max_host_offset()? {
                bail!("No IP addresses left for clients (server range ends at offset {new_end})");
            }

            self.server_ip_range = Some((new_start, new_end));
            self.client_ip_start = Some(new_client_start);
            self.data_mut()?.network.server_ip_range = [new_start, new_end];
            self.data_mut()?.network.client_ip_start = new_client_start;
            has_changes = true;

            messages.push(Message::info(format!(
                "Server IP range updated: offset {}-{} -> {new_start}-{new_end}",
                old_range.0, old_range.1
            )));
            messages.push(Message::info(format!(
                "Client IP start updated to offset: {new_client_start}"
            )));
        }

        if !has_changes {
            bail!("No changes specified.");
        }

        let backup = self.backup_data()?;
        messages.push(Message::warning(format!(
            "Old data backed up to: {}",
            backup.display()
        )));
        self.save_data()?;
        messages.push(Message::success("Configuration updated successfully!"));

        Ok(OperationResult {
            messages,
            backup_path: Some(backup),
        })
    }

    /// Add a new server, allocating the next available server IP and generating keys/PSKs.
    pub fn add_server(
        &mut self,
        name: &str,
        public_ip: &str,
        port: u16,
    ) -> Result<OperationResult> {
        self.ensure_loaded()?;

        if self.name_exists(name)? {
            bail!("Name '{name}' already exists");
        }

        let range = self
            .server_ip_range
            .ok_or_else(|| anyhow!("Server IP range not initialized"))?;
        let ip = self.get_next_server_ip()?.ok_or_else(|| {
            let max = range.1 - range.0 + 1;
            anyhow::anyhow!("Server IP exhausted (max {max})")
        })?;

        let keypair = generate_keypair();
        let mut preshared_keys = BTreeMap::new();

        // PSK with existing servers
        for (server_name, server) in &mut self.data_mut()?.servers {
            let psk = generate_preshared_key();
            preshared_keys.insert(server_name.clone(), psk.clone());
            server.preshared_keys.insert(name.to_string(), psk);
        }

        // PSK with existing clients
        for client_name in self.data()?.clients.keys() {
            preshared_keys.insert(client_name.clone(), generate_preshared_key());
        }

        let entry = ServerEntry {
            name: name.to_string(),
            ip: ip.clone(),
            public_ip: public_ip.to_string(),
            listen_port: port,
            private_key: keypair.private_key,
            public_key: keypair.public_key,
            preshared_keys,
        };

        self.data_mut()?.servers.insert(name.to_string(), entry);
        self.save_data()?;

        Ok(OperationResult::success(format!(
            "Server '{name}' added, IP: {ip}"
        )))
    }

    /// Add a new client, allocating the next available client IP and generating keys.
    pub fn add_client(&mut self, name: &str) -> Result<OperationResult> {
        self.ensure_loaded()?;

        if self.name_exists(name)? {
            bail!("Name '{name}' already exists");
        }

        let ip = self
            .get_next_client_ip()?
            .ok_or_else(|| anyhow::anyhow!("Client IP exhausted"))?;

        let keypair = generate_keypair();

        for server in self.data_mut()?.servers.values_mut() {
            server
                .preshared_keys
                .insert(name.to_string(), generate_preshared_key());
        }

        let entry = ClientEntry {
            name: name.to_string(),
            ip: ip.clone(),
            private_key: keypair.private_key,
            public_key: keypair.public_key,
        };

        self.data_mut()?.clients.insert(name.to_string(), entry);
        self.save_data()?;

        Ok(OperationResult::success(format!(
            "Client '{name}' added, IP: {ip}"
        )))
    }

    /// Edit a server's public IP and/or listen port.
    pub fn edit_server(
        &mut self,
        name: &str,
        public_ip: Option<&str>,
        port: Option<u16>,
    ) -> Result<OperationResult> {
        self.ensure_loaded()?;

        let server = self
            .data_mut()?
            .servers
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Server '{name}' not found"))?;

        if let Some(pip) = public_ip {
            server.public_ip = pip.to_string();
        }
        if let Some(p) = port {
            server.listen_port = p;
        }

        let modified = public_ip.is_some() || port.is_some();

        if modified {
            self.save_data()?;
            Ok(OperationResult::success(format!("Server '{name}' updated")))
        } else {
            Ok(OperationResult::success(format!(
                "No changes needed for server '{name}'"
            )))
        }
    }

    /// Edit (rename) a client, updating all server PSK references.
    pub fn edit_client(&mut self, name: &str, new_name: Option<&str>) -> Result<OperationResult> {
        self.ensure_loaded()?;

        if !self.data()?.clients.contains_key(name) {
            bail!("Client '{name}' not found");
        }

        if let Some(nn) = new_name {
            if self.name_exists(nn)? {
                bail!("Name '{nn}' already exists");
            }

            let mut client = self
                .data_mut()?
                .clients
                .remove(name)
                .ok_or_else(|| anyhow!("Client '{name}' not found during removal"))?;
            client.name = nn.to_string();
            self.data_mut()?.clients.insert(nn.to_string(), client);

            for server in self.data_mut()?.servers.values_mut() {
                if let Some(psk) = server.preshared_keys.remove(name) {
                    server.preshared_keys.insert(nn.to_string(), psk);
                }
            }

            self.save_data()?;
            return Ok(OperationResult::success(format!(
                "Client '{name}' renamed to '{nn}'"
            )));
        }

        Ok(OperationResult::success(format!(
            "No changes needed for client '{name}'"
        )))
    }

    /// Delete a server and remove its PSK entries from all other servers.
    pub fn delete_server(&mut self, name: &str) -> Result<OperationResult> {
        self.ensure_loaded()?;

        if !self.data()?.servers.contains_key(name) {
            bail!("Server '{name}' not found");
        }

        self.data_mut()?.servers.remove(name);

        for server in self.data_mut()?.servers.values_mut() {
            server.preshared_keys.remove(name);
        }

        self.save_data()?;
        Ok(OperationResult::success(format!("Server '{name}' deleted")))
    }

    /// Delete a client and remove its PSK entries from all servers.
    pub fn delete_client(&mut self, name: &str) -> Result<OperationResult> {
        self.ensure_loaded()?;

        if !self.data()?.clients.contains_key(name) {
            bail!("Client '{name}' not found");
        }

        self.data_mut()?.clients.remove(name);

        for server in self.data_mut()?.servers.values_mut() {
            server.preshared_keys.remove(name);
        }

        self.save_data()?;
        Ok(OperationResult::success(format!("Client '{name}' deleted")))
    }

    /// Return references to the server and client maps.
    pub fn list_devices(&self) -> Result<DeviceList<'_>> {
        self.ensure_loaded()?;
        let data = self.data()?;
        Ok((&data.servers, &data.clients))
    }

    /// Look up a device by name and return its details.
    pub fn show_device(&self, name: &str) -> Result<DeviceInfo> {
        self.ensure_loaded()?;

        match self.get_device_type(name)? {
            Some("server") => {
                let s = self
                    .data()?
                    .servers
                    .get(name)
                    .ok_or_else(|| anyhow!("Server '{name}' not found"))?;
                Ok(DeviceInfo::Server {
                    name: s.name.clone(),
                    ip: s.ip.clone(),
                    public_ip: s.public_ip.clone(),
                    listen_port: s.listen_port,
                    private_key: s.private_key.clone(),
                    public_key: s.public_key.clone(),
                    preshared_keys_count: s.preshared_keys.len(),
                })
            }
            Some("client") => {
                let c = self
                    .data()?
                    .clients
                    .get(name)
                    .ok_or_else(|| anyhow!("Client '{name}' not found"))?;
                Ok(DeviceInfo::Client {
                    name: c.name.clone(),
                    ip: c.ip.clone(),
                    private_key: c.private_key.clone(),
                    public_key: c.public_key.clone(),
                })
            }
            _ => bail!("Device '{name}' not found"),
        }
    }

    /// Generate a `WireGuard` configuration file for the named server.
    pub fn generate_server_config(&self, name: &str) -> Result<String> {
        self.ensure_loaded()?;

        let data = self.data()?;
        let server = data
            .servers
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Server '{name}' not found"))?;

        let prefixlen = self
            .network
            .ok_or_else(|| anyhow!("Network not initialized"))?
            .prefix_len();
        let mut lines = vec![
            "[Interface]".to_string(),
            format!("PrivateKey = {}", server.private_key),
            format!("Address = {}/{prefixlen}", server.ip),
            format!("ListenPort = {}", server.listen_port),
        ];

        for (other_name, other_server) in &data.servers {
            if other_name == name {
                continue;
            }
            let psk = server
                .preshared_keys
                .get(other_name)
                .map_or("", std::string::String::as_str);
            push_peer_section(
                &mut lines,
                other_name,
                "Server",
                &other_server.public_key,
                psk,
                &other_server.ip,
                Some((&other_server.public_ip, other_server.listen_port)),
                false,
            );
        }

        for (client_name, client) in &data.clients {
            let psk = server
                .preshared_keys
                .get(client_name)
                .map_or("", std::string::String::as_str);
            push_peer_section(
                &mut lines,
                client_name,
                "Client",
                &client.public_key,
                psk,
                &client.ip,
                None,
                false,
            );
        }

        Ok(lines.join("\n"))
    }

    /// Generate a `WireGuard` configuration file for the named client.
    pub fn generate_client_config(&self, name: &str) -> Result<String> {
        self.ensure_loaded()?;

        let data = self.data()?;
        let client = data
            .clients
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Client '{name}' not found"))?;

        let prefixlen = self
            .network
            .ok_or_else(|| anyhow!("Network not initialized"))?
            .prefix_len();
        let mut lines = vec![
            "[Interface]".to_string(),
            format!("PrivateKey = {}", client.private_key),
            format!("Address = {}/{prefixlen}", client.ip),
        ];

        for (server_name, server) in &data.servers {
            let psk = server
                .preshared_keys
                .get(name)
                .map_or("", std::string::String::as_str);
            push_peer_section(
                &mut lines,
                server_name,
                "Server",
                &server.public_key,
                psk,
                &server.ip,
                Some((&server.public_ip, server.listen_port)),
                true,
            );
        }

        Ok(lines.join("\n"))
    }

    /// Generate config for any device. Writes to `output_dir` if given, otherwise returns the content.
    pub fn generate_config(&self, name: &str, output_dir: Option<&str>) -> Result<Option<String>> {
        self.ensure_loaded()?;

        let config = match self.get_device_type(name)? {
            Some("server") => self.generate_server_config(name)?,
            Some("client") => self.generate_client_config(name)?,
            _ => bail!("Device '{name}' not found"),
        };

        if let Some(dir) = output_dir {
            let path = Path::new(dir);
            Self::save_config_file(name, &config, path)?;
            Ok(None)
        } else {
            Ok(Some(config))
        }
    }

    /// Generate and save config files for every device, returning the count written.
    pub fn generate_all_configs(&self, output_dir: &str) -> Result<u32> {
        self.ensure_loaded()?;
        let output_path = Path::new(output_dir);
        let mut count = 0u32;

        let server_names: Vec<String> = self.data()?.servers.keys().cloned().collect();
        for name in &server_names {
            let config = self.generate_server_config(name)?;
            Self::save_config_file(name, &config, output_path)?;
            count += 1;
        }

        let client_names: Vec<String> = self.data()?.clients.keys().cloned().collect();
        for name in &client_names {
            let config = self.generate_client_config(name)?;
            Self::save_config_file(name, &config, output_path)?;
            count += 1;
        }

        Ok(count)
    }
}

/// Append a `[Peer]` section to the config line buffer.
#[allow(clippy::too_many_arguments)]
fn push_peer_section(
    lines: &mut Vec<String>,
    peer_name: &str,
    peer_type: &str,
    public_key: &str,
    psk: &str,
    allowed_ip: &str,
    endpoint: Option<(&str, u16)>,
    keepalive: bool,
) {
    lines.push(String::new());
    lines.push(format!("# Peer: {peer_name} ({peer_type})"));
    lines.push("[Peer]".to_string());
    lines.push(format!("PublicKey = {public_key}"));
    lines.push(format!("PresharedKey = {psk}"));
    lines.push(format!("AllowedIPs = {allowed_ip}/32"));
    if let Some((ip, port)) = endpoint {
        lines.push(format!("Endpoint = {ip}:{port}"));
    }
    if keepalive {
        lines.push("PersistentKeepalive = 25".to_string());
    }
}

// ── Device info enum ─────────────────────────────────────────────────

/// Device details returned by `show_device`, used by both CLI and TUI.
#[derive(Clone)]
pub enum DeviceInfo {
    Server {
        name: String,
        ip: String,
        public_ip: String,
        listen_port: u16,
        private_key: String,
        public_key: String,
        preshared_keys_count: usize,
    },
    Client {
        name: String,
        ip: String,
        private_key: String,
        public_key: String,
    },
}

// ── Server range parser ──────────────────────────────────────────────

/// Parse a `[START,END]` string into a server offset range tuple.
pub fn parse_server_range(raw: &str) -> Result<(u32, u32)> {
    let inner = raw
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| anyhow::anyhow!("Invalid format: {raw}. Expected [START,END]"))?;
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        bail!("Expected exactly 2 values, got {}", parts.len());
    }
    let start: u32 = parts[0].trim().parse().context("Invalid start value")?;
    let end: u32 = parts[1].trim().parse().context("Invalid end value")?;
    Ok((start, end))
}

#[cfg(test)]
mod tests;
