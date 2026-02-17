//! Tests for data persistence across manager instances.

use super::*;

// ── Data persistence ─────────────────────────────────────────────

#[test]
fn test_data_persists_across_manager_instances() {
    let (tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_client("c1").unwrap();
    drop(mgr);

    let data_path = tmp.path().join("data.json");
    let mgr2 = WireGuardManager::new(data_path.to_str().unwrap());
    let (servers, clients) = mgr2.list_devices().unwrap();
    assert_eq!(servers.len(), 1);
    assert_eq!(clients.len(), 1);
    assert_eq!(servers["srv1"].ip, "10.0.0.1");
    assert_eq!(clients["c1"].ip, "10.0.0.11");
}

#[test]
fn test_data_file_is_valid_json() {
    let (tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();
    mgr.add_client("c1").unwrap();

    let content = fs::read_to_string(tmp.path().join("data.json")).unwrap();
    let data: DataFile = serde_json::from_str(&content).unwrap();
    assert_eq!(data.servers.len(), 1);
    assert_eq!(data.clients.len(), 1);
}

#[test]
fn test_backup_file_created_on_force_init() {
    let (tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820).unwrap();

    mgr.init_network("10.0.0.0/24", 1, 10, true).unwrap();

    let bak_files: Vec<_> = fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "bak"))
        .collect();
    assert_eq!(bak_files.len(), 1);

    // Backup should contain the old data with srv1
    let bak_content = fs::read_to_string(bak_files[0].path()).unwrap();
    let bak_data: DataFile = serde_json::from_str(&bak_content).unwrap();
    assert!(bak_data.servers.contains_key("srv1"));
}
