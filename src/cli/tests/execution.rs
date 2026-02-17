//! Integration tests for CLI `handle_*` execution functions.

use super::*;

// ── print_messages ───────────────────────────────────────────────

#[test]
fn print_messages_does_not_panic_on_success() {
    let result = OperationResult {
        messages: vec![manager::Message {
            kind: MessageKind::Success,
            text: "ok".to_string(),
        }],
        backup_path: None,
    };
    print_messages(&result);
}

#[test]
fn print_messages_does_not_panic_on_info_and_warning() {
    let result = OperationResult {
        messages: vec![
            manager::Message {
                kind: MessageKind::Info,
                text: "info line".to_string(),
            },
            manager::Message {
                kind: MessageKind::Warning,
                text: "warning line".to_string(),
            },
        ],
        backup_path: None,
    };
    print_messages(&result);
}

#[test]
fn print_messages_handles_empty_messages() {
    let result = OperationResult {
        messages: vec![],
        backup_path: None,
    };
    print_messages(&result);
}

// ── handle_init ──────────────────────────────────────────────────

#[test]
fn handle_init_succeeds() {
    let (_tmp, mut mgr) = setup_empty_manager();
    assert!(handle_init(&mut mgr, "10.0.0.0/24", "[1,10]", false).is_ok());
}

#[test]
fn handle_init_invalid_range_format() {
    let (_tmp, mut mgr) = setup_empty_manager();
    let err = handle_init(&mut mgr, "10.0.0.0/24", "bad_range", false).unwrap_err();
    assert!(err.to_string().contains("Invalid server range"));
}

#[test]
fn handle_init_invalid_subnet() {
    let (_tmp, mut mgr) = setup_empty_manager();
    assert!(handle_init(&mut mgr, "not-a-subnet", "[1,10]", false).is_err());
}

#[test]
fn handle_init_duplicate_without_force() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(handle_init(&mut mgr, "10.0.0.0/24", "[1,10]", false).is_err());
}

// ── handle_config ────────────────────────────────────────────────

#[test]
fn handle_config_show_succeeds() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(handle_config(&mut mgr, ConfigAction::Show).is_ok());
}

#[test]
fn handle_config_show_uninitialized() {
    let (_tmp, mut mgr) = setup_empty_manager();
    assert!(handle_config(&mut mgr, ConfigAction::Show).is_err());
}

#[test]
fn handle_config_set_succeeds() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        handle_config(
            &mut mgr,
            ConfigAction::Set {
                subnet: None,
                server_range: Some("[1,20]".to_string()),
            },
        )
        .is_ok()
    );
}

#[test]
fn handle_config_set_invalid_range() {
    let (_tmp, mut mgr) = setup_manager();
    let err = handle_config(
        &mut mgr,
        ConfigAction::Set {
            subnet: None,
            server_range: Some("bad".to_string()),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("Invalid server range"));
}

// ── handle_add ───────────────────────────────────────────────────

#[test]
fn handle_add_server_succeeds() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        handle_add(
            &mut mgr,
            AddDevice::Server {
                name: "s1".to_string(),
                public_ip: "1.2.3.4".to_string(),
                port: 51820,
            },
        )
        .is_ok()
    );
}

#[test]
fn handle_add_client_succeeds() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        handle_add(
            &mut mgr,
            AddDevice::Client {
                name: "c1".to_string()
            }
        )
        .is_ok()
    );
}

#[test]
fn handle_add_duplicate_name_fails() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    assert!(
        handle_add(
            &mut mgr,
            AddDevice::Server {
                name: "srv1".to_string(),
                public_ip: "5.6.7.8".to_string(),
                port: 51821,
            },
        )
        .is_err()
    );
}

// ── handle_edit ──────────────────────────────────────────────────

#[test]
fn handle_edit_server_port_succeeds() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    assert!(
        handle_edit(
            &mut mgr,
            EditDevice::Server {
                name: "srv1".to_string(),
                public_ip: None,
                port: Some(12345),
            },
        )
        .is_ok()
    );
}

#[test]
fn handle_edit_client_rename_succeeds() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    assert!(
        handle_edit(
            &mut mgr,
            EditDevice::Client {
                name: "client1".to_string(),
                new_name: Some("renamed".to_string()),
            },
        )
        .is_ok()
    );
}

#[test]
fn handle_edit_nonexistent_fails() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        handle_edit(
            &mut mgr,
            EditDevice::Server {
                name: "ghost".to_string(),
                public_ip: Some("1.1.1.1".to_string()),
                port: None,
            },
        )
        .is_err()
    );
}

// ── handle_del ───────────────────────────────────────────────────

#[test]
fn handle_del_server_succeeds() {
    let (_tmp, mut mgr) = setup_manager_with_devices();
    assert!(
        handle_del(
            &mut mgr,
            DelDevice::Server {
                name: "srv1".to_string()
            },
        )
        .is_ok()
    );
}

#[test]
fn handle_del_nonexistent_fails() {
    let (_tmp, mut mgr) = setup_manager();
    assert!(
        handle_del(
            &mut mgr,
            DelDevice::Client {
                name: "ghost".to_string()
            },
        )
        .is_err()
    );
}

// ── handle_gen ───────────────────────────────────────────────────

#[test]
fn handle_gen_single_to_stdout_succeeds() {
    let (_tmp, mgr) = setup_manager_with_devices();
    assert!(handle_gen(&mgr, "srv1", None).is_ok());
}

#[test]
fn handle_gen_all_succeeds() {
    let (tmp, mgr) = setup_manager_with_devices();
    let output_dir = tmp.path().join("configs");
    assert!(handle_gen(&mgr, "all", Some(output_dir.to_str().unwrap())).is_ok());
    assert!(output_dir.join("srv1.conf").exists());
    assert!(output_dir.join("client1.conf").exists());
}

#[test]
fn handle_gen_nonexistent_fails() {
    let (_tmp, mgr) = setup_manager();
    assert!(handle_gen(&mgr, "ghost", None).is_err());
}

// ── handle_list ──────────────────────────────────────────────────

#[test]
fn handle_list_with_devices_succeeds() {
    let (_tmp, mgr) = setup_manager_with_devices();
    assert!(handle_list(&mgr).is_ok());
}

#[test]
fn handle_list_empty_succeeds() {
    let (_tmp, mgr) = setup_manager();
    assert!(handle_list(&mgr).is_ok());
}

// ── handle_show ──────────────────────────────────────────────────

#[test]
fn handle_show_existing_device_succeeds() {
    let (_tmp, mgr) = setup_manager_with_devices();
    assert!(handle_show(&mgr, "srv1").is_ok());
    assert!(handle_show(&mgr, "client1").is_ok());
}

#[test]
fn handle_show_nonexistent_fails() {
    let (_tmp, mgr) = setup_manager();
    assert!(handle_show(&mgr, "ghost").is_err());
}
