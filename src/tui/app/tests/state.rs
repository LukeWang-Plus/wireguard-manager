//! Tests for confirm dialogs, edit/delete selection, and form guards.

use super::*;

// ── Key handling: confirm dialogs ────────────────────────────────────

#[test]
fn confirm_n_cancels() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.mode = Mode::ConfirmDelete {
        name: "srv1".to_string(),
        device_type: "server".to_string(),
    };
    app.handle_key(key_char('n'));
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn confirm_esc_cancels() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::ConfirmReinit;
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn confirm_reinit_y_opens_init_form_with_force() {
    let (_tmp, mut app) = setup_app_with_network();
    app.mode = Mode::ConfirmReinit;
    app.handle_key(key_char('y'));
    assert!(app.force_init);
    assert!(matches!(app.mode, Mode::InitNetwork));
    assert_eq!(app.form_labels.len(), 2); // Subnet, Server Range
}

#[test]
fn confirm_delete_y_deletes_server() {
    let (_tmp, mut app) = setup_app_with_devices();
    let initial_count = app.device_count();
    app.mode = Mode::ConfirmDelete {
        name: "srv1".to_string(),
        device_type: "server".to_string(),
    };
    app.handle_key(key_char('y'));
    assert!(matches!(app.mode, Mode::Normal));
    assert_eq!(app.device_count(), initial_count - 1);
    assert!(app.servers.is_empty());
}

#[test]
fn confirm_delete_y_deletes_client() {
    let (_tmp, mut app) = setup_app_with_devices();
    let initial_count = app.device_count();
    app.mode = Mode::ConfirmDelete {
        name: "client1".to_string(),
        device_type: "client".to_string(),
    };
    app.handle_key(key_char('y'));
    assert!(matches!(app.mode, Mode::Normal));
    assert_eq!(app.device_count(), initial_count - 1);
    assert!(app.clients.is_empty());
}

// ── Open edit / delete selected ──────────────────────────────────────

#[test]
fn device_list_e_opens_edit_for_server() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.device_index = 0; // server
    app.handle_key(key_char('e'));
    assert!(matches!(app.mode, Mode::EditServer { .. }));
}

#[test]
fn device_list_e_opens_edit_for_client() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.device_index = 1; // client
    app.handle_key(key_char('e'));
    assert!(matches!(app.mode, Mode::EditClient { .. }));
}

#[test]
fn device_list_d_opens_confirm_delete_for_server() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.device_index = 0;
    app.handle_key(key_char('d'));
    if let Mode::ConfirmDelete { name, device_type } = &app.mode {
        assert_eq!(name, "srv1");
        assert_eq!(device_type, "server");
    } else {
        panic!("Expected ConfirmDelete mode");
    }
}

#[test]
fn device_list_d_opens_confirm_delete_for_client() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.device_index = 1;
    app.handle_key(key_char('d'));
    if let Mode::ConfirmDelete { name, device_type } = &app.mode {
        assert_eq!(name, "client1");
        assert_eq!(device_type, "client");
    } else {
        panic!("Expected ConfirmDelete mode");
    }
}

#[test]
fn device_list_e_noop_when_empty() {
    let (_tmp, mut app) = setup_app_with_network();
    app.panel = Panel::DeviceList;
    app.handle_key(key_char('e'));
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn device_list_d_noop_when_empty() {
    let (_tmp, mut app) = setup_app_with_network();
    app.panel = Panel::DeviceList;
    app.handle_key(key_char('d'));
    assert!(matches!(app.mode, Mode::Normal));
}

// ── Add form: not initialized guard ──────────────────────────────────

#[test]
fn add_server_form_requires_initialized_network() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::ChooseDeviceType;
    app.handle_key(key_char('s'));
    // open_add_server_form checks network_info and sets status instead
    assert!(matches!(app.mode, Mode::Normal));
    assert!(app.status.contains("not initialized"));
}

#[test]
fn add_client_form_requires_initialized_network() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::ChooseDeviceType;
    app.handle_key(key_char('c'));
    assert!(matches!(app.mode, Mode::Normal));
    assert!(app.status.contains("not initialized"));
}

// ── handle_operation_result ──────────────────────────────────────────

#[test]
fn handle_operation_result_sets_status_from_success() {
    let (_tmp, mut app) = setup_app();
    let result = OperationResult {
        messages: vec![Message {
            kind: MessageKind::Success,
            text: "Operation completed".to_string(),
        }],
        backup_path: None,
    };
    app.handle_operation_result(&result);
    assert_eq!(app.status, "Operation completed");
}

#[test]
fn handle_operation_result_appends_to_logs() {
    let (_tmp, mut app) = setup_app();
    let result = OperationResult {
        messages: vec![
            Message {
                kind: MessageKind::Success,
                text: "done".to_string(),
            },
            Message {
                kind: MessageKind::Info,
                text: "details".to_string(),
            },
        ],
        backup_path: None,
    };
    app.handle_operation_result(&result);
    assert_eq!(app.logs.len(), 2);
    assert_eq!(app.logs[0].1, "done");
    assert_eq!(app.logs[1].1, "details");
}

#[test]
fn handle_operation_result_no_success_leaves_status_unchanged() {
    let (_tmp, mut app) = setup_app();
    app.status = "previous status".to_string();
    let result = OperationResult {
        messages: vec![Message {
            kind: MessageKind::Info,
            text: "just info".to_string(),
        }],
        backup_path: None,
    };
    app.handle_operation_result(&result);
    assert_eq!(app.status, "previous status");
    assert_eq!(app.logs.len(), 1);
}
