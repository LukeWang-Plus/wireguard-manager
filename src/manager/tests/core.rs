//! Tests for core manager functionality: parsing, construction, init, and network config.

use super::*;

// ── parse_server_range ───────────────────────────────────────────

#[test]
fn test_parse_server_range_valid() {
    assert_eq!(parse_server_range("[1,10]").unwrap(), (1, 10));
}

#[test]
fn test_parse_server_range_with_spaces() {
    assert_eq!(parse_server_range("[ 5 , 20 ]").unwrap(), (5, 20));
}

#[test]
fn test_parse_server_range_missing_brackets() {
    assert!(parse_server_range("1,10").is_err());
}

#[test]
fn test_parse_server_range_missing_end_bracket() {
    assert!(parse_server_range("[1,10").is_err());
}

#[test]
fn test_parse_server_range_wrong_count() {
    let err = parse_server_range("[1,2,3]").unwrap_err();
    assert!(err.to_string().contains("Expected exactly 2 values"));
}

#[test]
fn test_parse_server_range_non_numeric() {
    assert!(parse_server_range("[abc,10]").is_err());
}

// ── Constructor and loading ──────────────────────────────────────

#[test]
fn test_new_without_data_file() {
    let (_tmp, mgr) = setup_empty_manager();
    assert!(mgr.list_devices().is_err());
}

#[test]
fn test_new_loads_existing_data_file() {
    let (tmp, _mgr) = setup_manager();
    // Create a second manager pointing to the same data file
    let data_path = tmp.path().join("data.json");
    let mgr2 = WireGuardManager::new(data_path.to_str().unwrap());
    assert!(mgr2.list_devices().is_ok());
}

#[test]
fn test_new_with_corrupt_data_file() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    fs::write(&data_path, "not valid json!!!").unwrap();
    let mgr = WireGuardManager::new(data_path.to_str().unwrap());
    // load_data failed silently (let _ =), so data is None
    assert!(mgr.list_devices().is_err());
}

#[test]
fn test_ensure_loaded_fails_when_uninitialized() {
    let (_tmp, mgr) = setup_empty_manager();
    let err = mgr.list_devices().unwrap_err();
    assert!(err.to_string().contains("No configuration found"));
}

// ── init_network ─────────────────────────────────────────────────

#[test]
fn test_init_network_basic() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());

    let result = mgr.init_network("10.0.0.0/24", 1, 10, false).unwrap();
    assert!(
        result
            .messages
            .iter()
            .any(|m| m.text.contains("Network initialized successfully"))
    );
    assert!(data_path.exists());
}

#[test]
fn test_init_network_data_file_contents() {
    let (tmp, _mgr) = setup_manager();
    let content = fs::read_to_string(tmp.path().join("data.json")).unwrap();
    let data: DataFile = serde_json::from_str(&content).unwrap();
    assert_eq!(data.network.subnet, "10.0.0.0/24");
    assert_eq!(data.network.server_ip_range, [1, 10]);
    assert_eq!(data.network.client_ip_start, 11);
    assert!(data.servers.is_empty());
    assert!(data.clients.is_empty());
}

#[test]
fn test_init_network_refuses_without_force() {
    let (_tmp, mut mgr) = setup_manager();
    let err = mgr.init_network("10.0.0.0/24", 1, 10, false).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_init_network_force_reinitializes() {
    let (tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();

    let result = mgr.init_network("10.0.0.0/24", 1, 10, true).unwrap();
    assert!(result.backup_path.is_some());

    let (servers, clients) = mgr.list_devices().unwrap();
    assert!(servers.is_empty());
    assert!(clients.is_empty());

    // Check backup file exists
    let bak_files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "bak"))
        .collect();
    assert_eq!(bak_files.len(), 1);
}

#[test]
fn test_init_network_invalid_subnet() {
    let (_tmp, mut mgr) = setup_empty_manager();
    let err = mgr.init_network("not-a-subnet", 1, 10, false).unwrap_err();
    assert!(err.to_string().contains("Invalid subnet"));
}

#[test]
fn test_init_network_server_start_zero() {
    let (_tmp, mut mgr) = setup_empty_manager();
    let err = mgr.init_network("10.0.0.0/24", 0, 10, false).unwrap_err();
    assert!(err.to_string().contains("must be >= 1"));
}

#[test]
fn test_init_network_server_end_exceeds_max() {
    let (_tmp, mut mgr) = setup_empty_manager();
    let err = mgr.init_network("10.0.0.0/24", 1, 300, false).unwrap_err();
    assert!(err.to_string().contains("must be <="));
}

#[test]
fn test_init_network_start_greater_than_end() {
    let (_tmp, mut mgr) = setup_empty_manager();
    let err = mgr.init_network("10.0.0.0/24", 10, 5, false).unwrap_err();
    assert!(err.to_string().contains("must be <="));
}

// ── show_network_config ──────────────────────────────────────────

#[test]
fn test_show_network_config_basic() {
    let (_tmp, mgr) = setup_manager();
    let info = mgr.show_network_config().unwrap();

    assert_eq!(info.subnet, "10.0.0.0/24");
    assert_eq!(info.server_start_ip, Ipv4Addr::new(10, 0, 0, 1));
    assert_eq!(info.server_end_ip, Ipv4Addr::new(10, 0, 0, 10));
    assert_eq!(info.client_start_ip, Ipv4Addr::new(10, 0, 0, 11));
    assert_eq!(info.client_end_ip, Ipv4Addr::new(10, 0, 0, 254));
    assert_eq!(info.server_count, 0);
    assert_eq!(info.client_count, 0);
    assert_eq!(info.server_capacity, 10);
    assert_eq!(info.client_capacity, 244);
}

#[test]
fn test_show_network_config_with_devices() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("s1", "1.1.1.1", 51820).unwrap();
    mgr.add_server("s2", "2.2.2.2", 51820).unwrap();
    mgr.add_client("c1").unwrap();
    mgr.add_client("c2").unwrap();
    mgr.add_client("c3").unwrap();

    let info = mgr.show_network_config().unwrap();
    assert_eq!(info.server_count, 2);
    assert_eq!(info.client_count, 3);
}

#[test]
fn test_show_network_config_uninitialized() {
    let (_tmp, mgr) = setup_empty_manager();
    assert!(mgr.show_network_config().is_err());
}

// ── update_network_config ────────────────────────────────────────

#[test]
fn test_update_network_config_change_server_range() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.update_network_config(None, Some(1), Some(20)).unwrap();

    let info = mgr.show_network_config().unwrap();
    assert_eq!(info.server_end_ip, Ipv4Addr::new(10, 0, 0, 20));
    assert_eq!(info.client_start_offset, 21);
}

#[test]
fn test_update_network_config_change_subnet_empty_network() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.update_network_config(Some("192.168.0.0/24"), None, None)
        .unwrap();

    let info = mgr.show_network_config().unwrap();
    assert_eq!(info.subnet, "192.168.0.0/24");
}

#[test]
fn test_update_network_config_change_subnet_with_devices() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    let err = mgr
        .update_network_config(Some("192.168.0.0/24"), None, None)
        .unwrap_err();
    assert!(err.to_string().contains("Cannot change subnet"));
}

#[test]
fn test_update_network_config_no_changes() {
    let (_tmp, mut mgr) = setup_manager();
    let err = mgr.update_network_config(None, None, None).unwrap_err();
    assert!(err.to_string().contains("No changes specified"));
}

#[test]
fn test_update_network_config_range_excludes_server() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    // srv1 has IP 10.0.0.1 (offset 1), changing to [5,10] should fail
    let err = mgr
        .update_network_config(None, Some(5), Some(10))
        .unwrap_err();
    assert!(err.to_string().contains("out of range"));
}

#[test]
fn test_update_network_config_range_conflicts_with_client() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_client("c1").unwrap(); // IP 10.0.0.11 (offset 11)
    // Expanding server range to [1,15] makes client_start=16, but c1 is at offset 11
    let err = mgr
        .update_network_config(None, Some(1), Some(15))
        .unwrap_err();
    assert!(err.to_string().contains("conflict"));
}

#[test]
fn test_update_network_config_creates_backup() {
    let (tmp, mut mgr) = setup_manager();
    mgr.update_network_config(None, Some(1), Some(20)).unwrap();

    let bak_files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "bak"))
        .collect();
    assert_eq!(bak_files.len(), 1);
}

#[test]
fn test_update_network_config_uninitialized() {
    let (_tmp, mut mgr) = setup_empty_manager();
    assert!(mgr.update_network_config(None, Some(1), Some(20)).is_err());
}
