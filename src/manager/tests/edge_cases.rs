//! Tests for edge cases, boundary conditions, and additional coverage.

use super::*;

// ── Edge cases / boundary ────────────────────────────────────────

#[test]
fn test_tiny_network_slash30() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    // /30 => 4 addresses, 2 usable (offsets 1 and 2)
    mgr.init_network("10.0.0.0/30", 1, 1, false).unwrap();

    let result = mgr.add_server("s1", "1.1.1.1", 51820).unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.1"));

    let result = mgr.add_client("c1").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.2"));

    // No more room
    assert!(mgr.add_server("s2", "2.2.2.2", 51820).is_err());
    assert!(mgr.add_client("c2").is_err());
}

#[test]
fn test_no_client_space_when_server_range_fills_network() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    // /24 => max_host_offset=254, server range [1,254] leaves client_start=255 > 254
    let err = mgr.init_network("10.0.0.0/24", 1, 254, false).unwrap_err();
    assert!(err.to_string().contains("No IP addresses left for clients"));
}

#[test]
fn test_server_range_single_offset() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    mgr.init_network("10.0.0.0/24", 5, 5, false).unwrap();

    let result = mgr.add_server("s1", "1.1.1.1", 51820).unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.5"));

    let err = mgr.add_server("s2", "2.2.2.2", 51820).unwrap_err();
    assert!(err.to_string().contains("Server IP exhausted"));
}

// ── Tests from Python test report + additional coverage ─────────

#[test]
fn test_init_network_slash31_too_small() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    let err = mgr.init_network("10.0.0.0/31", 1, 1, false).unwrap_err();
    assert!(err.to_string().contains("too small"));
}

#[test]
fn test_init_network_slash32_too_small() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    let err = mgr.init_network("10.0.0.0/32", 1, 1, false).unwrap_err();
    assert!(err.to_string().contains("too small"));
}

#[test]
fn test_add_client_name_conflicts_with_server() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("device1", "1.2.3.4", 51820).unwrap();
    let err = mgr.add_client("device1").unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_edit_client_rename_to_server_name() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_client("c1").unwrap();
    let err = mgr.edit_client("c1", Some("srv1")).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_client_ip_reuse_non_sequential() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_client("c1").unwrap(); // 10.0.0.11
    mgr.add_client("c2").unwrap(); // 10.0.0.12
    mgr.add_client("c3").unwrap(); // 10.0.0.13

    mgr.delete_client("c2").unwrap(); // free 10.0.0.12

    let result = mgr.add_client("c4").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.12")); // reuses lowest available
}

#[test]
fn test_server_ip_reuse_non_sequential() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("s1", "1.1.1.1", 51820).unwrap(); // 10.0.0.1
    mgr.add_server("s2", "2.2.2.2", 51821).unwrap(); // 10.0.0.2
    mgr.add_server("s3", "3.3.3.3", 51822).unwrap(); // 10.0.0.3

    mgr.delete_server("s2").unwrap(); // free 10.0.0.2

    let result = mgr.add_server("s4", "4.4.4.4", 51823).unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.2")); // reuses lowest available
}

#[test]
fn test_ip_exhaustion_small_network_slash29() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    // /29 => 8 addresses, 6 usable (offsets 1..6)
    mgr.init_network("10.0.0.0/29", 1, 2, false).unwrap();

    // Fill server slots
    assert!(
        mgr.add_server("s1", "1.1.1.1", 51820)
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.1")
    );
    assert!(
        mgr.add_server("s2", "2.2.2.2", 51821)
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.2")
    );
    let err = mgr.add_server("s3", "3.3.3.3", 51822).unwrap_err();
    assert!(err.to_string().contains("Server IP exhausted"));

    // Fill client slots (offsets 3..6 = 4 slots)
    assert!(
        mgr.add_client("c1")
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.3")
    );
    assert!(
        mgr.add_client("c2")
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.4")
    );
    assert!(
        mgr.add_client("c3")
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.5")
    );
    assert!(
        mgr.add_client("c4")
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.6")
    );
    let err = mgr.add_client("c5").unwrap_err();
    assert!(err.to_string().contains("Client IP exhausted"));

    // Verify capacity
    let info = mgr.show_network_config().unwrap();
    assert_eq!(info.server_count, 2);
    assert_eq!(info.server_capacity, 2);
    assert_eq!(info.client_count, 4);
    assert_eq!(info.client_capacity, 4);
}

#[test]
fn test_psk_symmetry_in_generated_configs() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_client("c1").unwrap();

    let srv_config = mgr.generate_server_config("srv1").unwrap();
    let cli_config = mgr.generate_client_config("c1").unwrap();

    // Extract PresharedKey from server config (for client peer)
    let srv_psk = srv_config
        .lines()
        .find(|l| l.starts_with("PresharedKey = "))
        .unwrap()
        .trim_start_matches("PresharedKey = ");

    // Extract PresharedKey from client config (for server peer)
    let cli_psk = cli_config
        .lines()
        .find(|l| l.starts_with("PresharedKey = "))
        .unwrap()
        .trim_start_matches("PresharedKey = ");

    assert_eq!(srv_psk, cli_psk);
}

#[test]
fn test_psk_symmetry_in_server_to_server_configs() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_server("srv2", "5.6.7.8", 51821).unwrap();

    let srv1_config = mgr.generate_server_config("srv1").unwrap();
    let srv2_config = mgr.generate_server_config("srv2").unwrap();

    // Extract PresharedKey from srv1 config (peer srv2)
    let srv1_psk = srv1_config
        .lines()
        .find(|l| l.starts_with("PresharedKey = "))
        .unwrap()
        .trim_start_matches("PresharedKey = ");

    // Extract PresharedKey from srv2 config (peer srv1)
    let srv2_psk = srv2_config
        .lines()
        .find(|l| l.starts_with("PresharedKey = "))
        .unwrap()
        .trim_start_matches("PresharedKey = ");

    assert_eq!(srv1_psk, srv2_psk);
}

#[test]
fn test_generate_client_config_no_listen_port() {
    let (_tmp, mgr) = setup_manager_with_devices();
    let config = mgr.generate_client_config("client1").unwrap();
    assert!(!config.contains("ListenPort"));
}

#[test]
fn test_generate_server_config_no_peers() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    let config = mgr.generate_server_config("srv1").unwrap();

    assert!(config.contains("[Interface]"));
    assert!(config.contains("ListenPort = 51820"));
    assert!(!config.contains("[Peer]"));
}

#[test]
fn test_generate_config_server_to_file() {
    let (tmp, mgr) = setup_manager_with_devices();
    let output_dir = tmp.path().join("output");
    let result = mgr
        .generate_config("srv1", Some(output_dir.to_str().unwrap()))
        .unwrap();
    assert!(result.is_none()); // written to file, not returned
    let conf_path = output_dir.join("srv1.conf");
    assert!(conf_path.exists());
    let content = fs::read_to_string(conf_path).unwrap();
    assert!(content.contains("[Interface]"));
    assert!(content.contains("ListenPort = 51820"));
}

#[test]
fn test_generate_all_configs_file_contents() {
    let (tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_client("c1").unwrap();

    let output_dir = tmp.path().join("configs");
    mgr.generate_all_configs(output_dir.to_str().unwrap())
        .unwrap();

    let srv_content = fs::read_to_string(output_dir.join("srv1.conf")).unwrap();
    assert!(srv_content.contains("[Interface]"));
    assert!(srv_content.contains("Address = 10.0.0.1/24"));
    assert!(srv_content.contains("ListenPort = 51820"));
    assert!(srv_content.contains("# Peer: c1 (Client)"));

    let cli_content = fs::read_to_string(output_dir.join("c1.conf")).unwrap();
    assert!(cli_content.contains("[Interface]"));
    assert!(cli_content.contains("Address = 10.0.0.11/24"));
    assert!(cli_content.contains("# Peer: srv1 (Server)"));
    assert!(cli_content.contains("PersistentKeepalive = 25"));
}

#[test]
fn test_update_network_config_invalid_subnet() {
    let (_tmp, mut mgr) = setup_manager();
    let err = mgr
        .update_network_config(Some("not-a-subnet"), None, None)
        .unwrap_err();
    assert!(err.to_string().contains("Invalid subnet"));
}

#[test]
fn test_multiple_ip_reuse_cycles() {
    let (_tmp, mut mgr) = setup_manager();

    // Cycle 1: add and delete
    let result = mgr.add_client("c1").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.11"));
    mgr.delete_client("c1").unwrap();

    // Cycle 2: reuses same IP
    let result = mgr.add_client("c2").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.11"));
    mgr.delete_client("c2").unwrap();

    // Cycle 3: still reuses
    let result = mgr.add_client("c3").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.11"));
}
