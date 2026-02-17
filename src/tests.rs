//! Shared test helpers and constants used across multiple test modules.

pub(crate) use tempfile::TempDir;

pub(crate) use crate::manager::WireGuardManager;

pub(crate) const TEST_SUBNET: &str = "10.0.0.0/24";
pub(crate) const TEST_SERVER_START: u32 = 1;
pub(crate) const TEST_SERVER_END: u32 = 10;

/// Creates an uninitialized manager (no data.json).
pub(crate) fn setup_empty_manager() -> (TempDir, WireGuardManager) {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let data_path = tmp.path().join("data.json");
    let data_str = data_path.to_str().expect("Non-UTF-8 temp path");
    let mgr = WireGuardManager::new(data_str);
    (tmp, mgr)
}

/// Creates a temp directory with an initialized `WireGuardManager` (/24, range [1,10]).
pub(crate) fn setup_manager() -> (TempDir, WireGuardManager) {
    let (tmp, mut mgr) = setup_empty_manager();
    mgr.init_network(TEST_SUBNET, TEST_SERVER_START, TEST_SERVER_END, false)
        .expect("Failed to init network");
    (tmp, mgr)
}

/// Creates an initialized manager with 1 server ("srv1") and 1 client ("client1").
pub(crate) fn setup_manager_with_devices() -> (TempDir, WireGuardManager) {
    let (tmp, mut mgr) = setup_manager();
    mgr.add_server("srv1", "1.2.3.4", 51820)
        .expect("add server");
    mgr.add_client("client1").expect("add client");
    (tmp, mgr)
}
