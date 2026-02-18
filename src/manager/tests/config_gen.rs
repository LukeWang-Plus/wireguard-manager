//! Tests for device listing, showing, and config generation.

use super::*;

// ── list_devices and show_device ─────────────────────────────────

#[test]
fn test_list_devices_empty() {
    let (_tmp, mgr) = setup_manager();
    let (servers, clients) = mgr.list_devices().unwrap();
    assert!(servers.is_empty());
    assert!(clients.is_empty());
}

#[test]
fn test_list_devices_populated() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.1.1.1", 51820).unwrap();
    mgr.add_server("srv2", "2.2.2.2", 51821).unwrap();
    mgr.add_client("c1").unwrap();
    mgr.add_client("c2").unwrap();

    let (servers, clients) = mgr.list_devices().unwrap();
    assert_eq!(servers.len(), 2);
    assert_eq!(clients.len(), 2);
    assert!(servers.contains_key("srv1"));
    assert!(servers.contains_key("srv2"));
    assert!(clients.contains_key("c1"));
    assert!(clients.contains_key("c2"));
}

#[test]
fn test_show_device_server() {
    let (_tmp, mgr) = setup_manager_with_devices();
    match mgr.show_device("srv1").unwrap() {
        DeviceInfo::Server {
            name,
            ip,
            public_ip,
            listen_port,
            ..
        } => {
            assert_eq!(name, "srv1");
            assert_eq!(ip, "10.0.0.1");
            assert_eq!(public_ip, "1.2.3.4");
            assert_eq!(listen_port, 51820);
        }
        DeviceInfo::Client { .. } => panic!("Expected Server variant"),
    }
}

#[test]
fn test_show_device_client() {
    let (_tmp, mgr) = setup_manager_with_devices();
    match mgr.show_device("client1").unwrap() {
        DeviceInfo::Client { name, ip, .. } => {
            assert_eq!(name, "client1");
            assert_eq!(ip, "10.0.0.11");
        }
        DeviceInfo::Server { .. } => panic!("Expected Client variant"),
    }
}

#[test]
fn test_show_device_not_found() {
    let (_tmp, mgr) = setup_manager();
    assert!(mgr.show_device("nonexistent").is_err());
}

// ── generate_server_config ───────────────────────────────────────

#[test]
fn test_generate_server_config_basic() {
    let (_tmp, mgr) = setup_manager_with_devices();
    let config = mgr.generate_server_config("srv1").unwrap();

    assert!(config.starts_with("[Interface]"));
    assert!(config.contains("PrivateKey = "));
    assert!(config.contains("Address = 10.0.0.1/24"));
    assert!(config.contains("ListenPort = 51820"));
}

#[test]
fn test_generate_server_config_with_peers() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_server("srv2", "5.6.7.8", 51821).unwrap();
    mgr.add_client("c1").unwrap();

    let config = mgr.generate_server_config("srv1").unwrap();

    // Should have peer section for srv2 (server)
    assert!(config.contains("# Peer: srv2 (Server)"));
    assert!(config.contains("Endpoint = 5.6.7.8:51821"));
    assert!(config.contains("AllowedIPs = 10.0.0.2/32"));

    // Should have peer section for c1 (client, no Endpoint)
    assert!(config.contains("# Peer: c1 (Client)"));
    assert!(config.contains("AllowedIPs = 10.0.0.11/32"));

    // Should NOT have self as peer
    assert!(!config.contains("# Peer: srv1"));
}

#[test]
fn test_generate_server_config_not_found() {
    let (_tmp, mgr) = setup_manager();
    assert!(mgr.generate_server_config("nonexistent").is_err());
}

// ── generate_client_config ───────────────────────────────────────

#[test]
fn test_generate_client_config_basic() {
    let (_tmp, mgr) = setup_manager_with_devices();
    let config = mgr.generate_client_config("client1").unwrap();

    assert!(config.starts_with("[Interface]"));
    assert!(config.contains("PrivateKey = "));
    assert!(config.contains("Address = 10.0.0.11/24"));
    assert!(config.contains("# Peer: srv1 (Server)"));
    assert!(config.contains("Endpoint = 1.2.3.4:51820"));
    assert!(config.contains("PersistentKeepalive = 25"));
}

#[test]
fn test_generate_client_config_multiple_servers() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_server("srv2", "5.6.7.8", 51821).unwrap();
    mgr.add_client("c1").unwrap();

    let config = mgr.generate_client_config("c1").unwrap();
    assert!(config.contains("# Peer: srv1 (Server)"));
    assert!(config.contains("# Peer: srv2 (Server)"));
}

#[test]
fn test_generate_client_config_not_found() {
    let (_tmp, mgr) = setup_manager();
    assert!(mgr.generate_client_config("nonexistent").is_err());
}

// ── generate_config ──────────────────────────────────────────────

#[test]
fn test_generate_config_server_to_stdout() {
    let (_tmp, mgr) = setup_manager_with_devices();
    let result = mgr.generate_config("srv1", None).unwrap();
    assert!(result.is_some());
    assert!(result.unwrap().contains("[Interface]"));
}

#[test]
fn test_generate_config_client_to_file() {
    let (tmp, mgr) = setup_manager_with_devices();
    let output_dir = tmp.path().join("output");
    let result = mgr
        .generate_config("client1", Some(output_dir.to_str().unwrap()))
        .unwrap();
    assert!(result.is_none());
    assert!(output_dir.join("client1.conf").exists());
}

#[test]
fn test_generate_config_not_found() {
    let (_tmp, mgr) = setup_manager();
    assert!(mgr.generate_config("nonexistent", None).is_err());
}

// ── generate_all_configs ─────────────────────────────────────────

#[test]
fn test_generate_all_configs_basic() {
    let (tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_server("srv2", "5.6.7.8", 51821).unwrap();
    mgr.add_client("c1").unwrap();
    mgr.add_client("c2").unwrap();

    let output_dir = tmp.path().join("configs");
    let count = mgr
        .generate_all_configs(output_dir.to_str().unwrap())
        .unwrap();
    assert_eq!(count, 4);
    assert!(output_dir.join("srv1.conf").exists());
    assert!(output_dir.join("srv2.conf").exists());
    assert!(output_dir.join("c1.conf").exists());
    assert!(output_dir.join("c2.conf").exists());
}

#[test]
fn test_generate_all_configs_empty_network() {
    let (tmp, mgr) = setup_manager();
    let output_dir = tmp.path().join("configs");
    let count = mgr
        .generate_all_configs(output_dir.to_str().unwrap())
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_generate_all_configs_creates_directory() {
    let (tmp, mgr) = setup_manager_with_devices();
    let output_dir = tmp.path().join("nested").join("output");
    assert!(!output_dir.exists());

    mgr.generate_all_configs(output_dir.to_str().unwrap())
        .unwrap();
    assert!(output_dir.exists());
}
