//! Tests for device management: add, edit, and delete servers/clients.

use super::*;

// ── add_server ───────────────────────────────────────────────────

#[test]
fn test_add_server_basic() {
    let (_tmp, mut mgr) = setup_manager();
    let result = mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.1"));

    let (servers, _) = mgr.list_devices().unwrap();
    assert_eq!(servers.len(), 1);
    let srv = &servers["srv1"];
    assert_eq!(srv.name, "srv1");
    assert_eq!(srv.ip, "10.0.0.1");
    assert_eq!(srv.public_ip, "1.2.3.4");
    assert_eq!(srv.listen_port, 51820);
}

#[test]
fn test_add_server_sequential_ip_allocation() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        mgr.add_server("s1", "1.1.1.1", 51820)
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.1")
    );
    assert!(
        mgr.add_server("s2", "2.2.2.2", 51820)
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.2")
    );
    assert!(
        mgr.add_server("s3", "3.3.3.3", 51820)
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.3")
    );
}

#[test]
fn test_add_server_duplicate_name() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    let err = mgr.add_server("srv1", "5.6.7.8", 51821).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_add_server_name_conflicts_with_client() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_client("device1").unwrap();
    let err = mgr.add_server("device1", "1.2.3.4", 51820).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_add_server_ip_exhaustion() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    mgr.init_network("10.0.0.0/24", 1, 2, false).unwrap();

    mgr.add_server("s1", "1.1.1.1", 51820).unwrap();
    mgr.add_server("s2", "2.2.2.2", 51820).unwrap();
    let err = mgr.add_server("s3", "3.3.3.3", 51820).unwrap_err();
    assert!(err.to_string().contains("Server IP exhausted"));
}

#[test]
fn test_add_server_generates_psk_with_existing_devices() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_client("c1").unwrap();
    mgr.add_server("srv2", "5.6.7.8", 51821).unwrap();

    let (servers, _) = mgr.list_devices().unwrap();
    let srv1 = &servers["srv1"];
    let srv2 = &servers["srv2"];

    // srv2 should have PSK for srv1 and c1
    assert!(srv2.preshared_keys.contains_key("srv1"));
    assert!(srv2.preshared_keys.contains_key("c1"));

    // srv1 should have PSK for srv2 (bidirectional)
    assert!(srv1.preshared_keys.contains_key("srv2"));

    // The PSK between srv1 and srv2 should be the same value
    assert_eq!(srv1.preshared_keys["srv2"], srv2.preshared_keys["srv1"]);
}

// ── add_client ───────────────────────────────────────────────────

#[test]
fn test_add_client_basic() {
    let (_tmp, mut mgr) = setup_manager();
    let result = mgr.add_client("c1").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.11"));

    let (_, clients) = mgr.list_devices().unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients["c1"].name, "c1");
}

#[test]
fn test_add_client_sequential_ip_allocation() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        mgr.add_client("c1")
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.11")
    );
    assert!(
        mgr.add_client("c2")
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.12")
    );
    assert!(
        mgr.add_client("c3")
            .unwrap()
            .success_text()
            .unwrap()
            .contains("10.0.0.13")
    );
}

#[test]
fn test_add_client_duplicate_name() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_client("c1").unwrap();
    let err = mgr.add_client("c1").unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_add_client_ip_exhaustion() {
    let tmp = TempDir::new().unwrap();
    let data_path = tmp.path().join("data.json");
    let mut mgr = WireGuardManager::new(data_path.to_str().unwrap());
    // /30 => 2 usable hosts (offsets 1 and 2), server range [1,1], client starts at 2
    mgr.init_network("10.0.0.0/30", 1, 1, false).unwrap();

    mgr.add_client("c1").unwrap(); // offset 2 (the only client slot)
    let err = mgr.add_client("c2").unwrap_err();
    assert!(err.to_string().contains("Client IP exhausted"));
}

#[test]
fn test_add_client_creates_psk_on_existing_servers() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_client("c1").unwrap();

    let (servers, _) = mgr.list_devices().unwrap();
    assert!(servers["srv1"].preshared_keys.contains_key("c1"));
}

// ── edit_server ──────────────────────────────────────────────────

#[test]
fn test_edit_server_change_public_ip() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    let result = mgr.edit_server("srv1", Some("9.8.7.6"), None).unwrap();
    assert!(result.success_text().unwrap().contains("updated"));

    if let DeviceInfo::Server { public_ip, .. } = mgr.show_device("srv1").unwrap() {
        assert_eq!(public_ip, "9.8.7.6");
    } else {
        panic!("Expected Server variant");
    }
}

#[test]
fn test_edit_server_change_port() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    let result = mgr.edit_server("srv1", None, Some(12345)).unwrap();
    assert!(result.success_text().unwrap().contains("updated"));

    if let DeviceInfo::Server { listen_port, .. } = mgr.show_device("srv1").unwrap() {
        assert_eq!(listen_port, 12345);
    } else {
        panic!("Expected Server variant");
    }
}

#[test]
fn test_edit_server_change_both() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    let result = mgr
        .edit_server("srv1", Some("9.8.7.6"), Some(12345))
        .unwrap();
    assert!(result.success_text().unwrap().contains("updated"));

    if let DeviceInfo::Server {
        public_ip,
        listen_port,
        ..
    } = mgr.show_device("srv1").unwrap()
    {
        assert_eq!(public_ip, "9.8.7.6");
        assert_eq!(listen_port, 12345);
    } else {
        panic!("Expected Server variant");
    }
}

#[test]
fn test_edit_server_no_changes() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    let result = mgr.edit_server("srv1", None, None).unwrap();
    assert!(result.success_text().unwrap().contains("No changes"));
}

#[test]
fn test_edit_server_not_found() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        mgr.edit_server("nonexistent", Some("1.1.1.1"), None)
            .is_err()
    );
}

// ── edit_client ──────────────────────────────────────────────────

#[test]
fn test_edit_client_rename() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    let result = mgr.edit_client("client1", Some("client2")).unwrap();
    assert!(result.success_text().unwrap().contains("renamed"));

    let (_, clients) = mgr.list_devices().unwrap();
    assert!(!clients.contains_key("client1"));
    assert!(clients.contains_key("client2"));
    assert_eq!(clients["client2"].ip, "10.0.0.11"); // IP preserved
}

#[test]
fn test_edit_client_rename_updates_psk_keys() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    // Get original PSK value
    let original_psk = {
        let (servers, _) = mgr.list_devices().unwrap();
        servers["srv1"].preshared_keys["client1"].clone()
    };

    mgr.edit_client("client1", Some("client2")).unwrap();

    let (servers, _) = mgr.list_devices().unwrap();
    assert!(!servers["srv1"].preshared_keys.contains_key("client1"));
    assert!(servers["srv1"].preshared_keys.contains_key("client2"));
    // PSK value preserved
    assert_eq!(servers["srv1"].preshared_keys["client2"], original_psk);
}

#[test]
fn test_edit_client_rename_to_existing_name() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_client("c1").unwrap();
    mgr.add_client("c2").unwrap();
    let err = mgr.edit_client("c1", Some("c2")).unwrap_err();
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn test_edit_client_no_changes() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    let result = mgr.edit_client("client1", None).unwrap();
    assert!(result.success_text().unwrap().contains("No changes"));
}

#[test]
fn test_edit_client_not_found() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(mgr.edit_client("nonexistent", Some("x")).is_err());
}

// ── delete_server ────────────────────────────────────────────────

#[test]
fn test_delete_server_basic() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    mgr.delete_server("srv1").unwrap();
    let (servers, _) = mgr.list_devices().unwrap();
    assert!(servers.is_empty());
}

#[test]
fn test_delete_server_removes_psk_from_other_servers() {
    let (_tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_server("srv2", "5.6.7.8", 51821).unwrap();

    // Verify srv1 has PSK for srv2
    let (servers, _) = mgr.list_devices().unwrap();
    assert!(servers["srv1"].preshared_keys.contains_key("srv2"));

    mgr.delete_server("srv2").unwrap();

    let (servers, _) = mgr.list_devices().unwrap();
    assert!(!servers["srv1"].preshared_keys.contains_key("srv2"));
}

#[test]
fn test_delete_server_not_found() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(mgr.delete_server("nonexistent").is_err());
}

#[test]
fn test_delete_server_ip_reuse() {
    let (_tmp, mut mgr) = setup_manager();
    let result = mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.1"));

    mgr.delete_server("srv1").unwrap();
    let result = mgr.add_server("srv2", "5.6.7.8", 51821).unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.1")); // IP recycled
}

// ── delete_client ────────────────────────────────────────────────

#[test]
fn test_delete_client_basic() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    mgr.delete_client("client1").unwrap();
    let (_, clients) = mgr.list_devices().unwrap();
    assert!(clients.is_empty());
}

#[test]
fn test_delete_client_removes_psk_from_servers() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    // Verify srv1 has PSK for client1
    let (servers, _) = mgr.list_devices().unwrap();
    assert!(servers["srv1"].preshared_keys.contains_key("client1"));

    mgr.delete_client("client1").unwrap();

    let (servers, _) = mgr.list_devices().unwrap();
    assert!(!servers["srv1"].preshared_keys.contains_key("client1"));
}

#[test]
fn test_delete_client_not_found() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(mgr.delete_client("nonexistent").is_err());
}

#[test]
fn test_delete_client_ip_reuse() {
    let (_tmp, mut mgr) = setup_manager();
    let result = mgr.add_client("c1").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.11"));

    mgr.delete_client("c1").unwrap();
    let result = mgr.add_client("c2").unwrap();
    assert!(result.success_text().unwrap().contains("10.0.0.11")); // IP recycled
}
