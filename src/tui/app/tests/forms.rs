//! Tests for form navigation and form submission.

use super::*;

// ── Key handling: form navigation ────────────────────────────────────

#[test]
fn form_tab_cycles_forward() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::new(), TextInput::new()];
    app.form_labels = vec!["F1", "F2"];
    app.form_focus = FormFocus::Field(0);
    app.mode = Mode::InitNetwork;

    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.form_focus, FormFocus::Field(1));

    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.form_focus, FormFocus::Submit);

    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.form_focus, FormFocus::Cancel);

    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.form_focus, FormFocus::Field(0));
}

#[test]
fn form_backtab_cycles_backward() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::new(), TextInput::new()];
    app.form_labels = vec!["F1", "F2"];
    app.form_focus = FormFocus::Field(0);
    app.mode = Mode::InitNetwork;

    app.handle_key(key(KeyCode::BackTab));
    assert_eq!(app.form_focus, FormFocus::Cancel);

    app.handle_key(key(KeyCode::BackTab));
    assert_eq!(app.form_focus, FormFocus::Submit);

    app.handle_key(key(KeyCode::BackTab));
    assert_eq!(app.form_focus, FormFocus::Field(1));

    app.handle_key(key(KeyCode::BackTab));
    assert_eq!(app.form_focus, FormFocus::Field(0));
}

#[test]
fn form_esc_cancels() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::InitNetwork;
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn form_enter_on_cancel_returns_to_normal() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::new()];
    app.mode = Mode::InitNetwork;
    app.form_focus = FormFocus::Cancel;
    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn form_enter_on_field_advances_to_next_field() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::new(), TextInput::new()];
    app.form_labels = vec!["F1", "F2"];
    app.form_focus = FormFocus::Field(0);
    app.mode = Mode::InitNetwork;
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.form_focus, FormFocus::Field(1));
}

#[test]
fn form_enter_on_last_field_advances_to_submit() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::new(), TextInput::new()];
    app.form_labels = vec!["F1", "F2"];
    app.form_focus = FormFocus::Field(1);
    app.mode = Mode::InitNetwork;
    app.handle_key(key(KeyCode::Enter));
    assert_eq!(app.form_focus, FormFocus::Submit);
}

#[test]
fn form_char_input_updates_field() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::new()];
    app.form_labels = vec!["F1"];
    app.form_focus = FormFocus::Field(0);
    app.mode = Mode::InitNetwork;
    app.handle_key(key_char('x'));
    app.handle_key(key_char('y'));
    assert_eq!(app.form_fields[0].value(), "xy");
}

#[test]
fn form_ctrl_u_clears_field() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::with_value("hello")];
    app.form_labels = vec!["F1"];
    app.form_focus = FormFocus::Field(0);
    app.mode = Mode::InitNetwork;
    app.handle_key(key_ctrl('u'));
    assert_eq!(app.form_fields[0].value(), "");
}

#[test]
fn form_backspace_deletes_char() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![TextInput::with_value("abc")];
    app.form_labels = vec!["F1"];
    app.form_focus = FormFocus::Field(0);
    app.mode = Mode::InitNetwork;
    app.handle_key(key(KeyCode::Backspace));
    assert_eq!(app.form_fields[0].value(), "ab");
}

// ── Form submission: integration ─────────────────────────────────────

#[test]
fn submit_init_form_initializes_network() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![
        TextInput::with_value("10.0.0.0/24"),
        TextInput::with_value("[1,10]"),
    ];
    app.form_labels = vec!["Subnet", "Server Range"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::InitNetwork;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::Normal));
    assert!(app.network_info.is_some());
}

#[test]
fn submit_init_form_invalid_range_stays_in_form() {
    let (_tmp, mut app) = setup_app();
    app.form_fields = vec![
        TextInput::with_value("10.0.0.0/24"),
        TextInput::with_value("invalid"),
    ];
    app.form_labels = vec!["Subnet", "Server Range"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::InitNetwork;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::InitNetwork));
    assert!(app.status.contains("Invalid server range"));
}

#[test]
fn submit_add_server_form() {
    let (_tmp, mut app) = setup_app_with_network();
    app.form_fields = vec![
        TextInput::with_value("myserver"),
        TextInput::with_value("1.2.3.4"),
        TextInput::with_value("51820"),
    ];
    app.form_labels = vec!["Name", "Public IP", "Port"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::AddServer;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::Normal));
    assert_eq!(app.servers.len(), 1);
    assert_eq!(app.servers[0].name, "myserver");
}

#[test]
fn submit_add_server_form_invalid_port_stays() {
    let (_tmp, mut app) = setup_app_with_network();
    app.form_fields = vec![
        TextInput::with_value("myserver"),
        TextInput::with_value("1.2.3.4"),
        TextInput::with_value("not_a_port"),
    ];
    app.form_labels = vec!["Name", "Public IP", "Port"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::AddServer;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::AddServer));
    assert!(app.status.contains("Invalid port"));
}

#[test]
fn submit_add_client_form() {
    let (_tmp, mut app) = setup_app_with_network();
    app.form_fields = vec![TextInput::with_value("myclient")];
    app.form_labels = vec!["Name"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::AddClient;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::Normal));
    assert_eq!(app.clients.len(), 1);
    assert_eq!(app.clients[0].name, "myclient");
}

#[test]
fn submit_edit_network_form() {
    let (_tmp, mut app) = setup_app_with_network();
    app.form_fields = vec![
        TextInput::with_value("10.0.0.0/16"),
        TextInput::with_value("[1,20]"),
    ];
    app.form_labels = vec!["Subnet", "Server Range"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::EditNetworkConfig;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::Normal));
    let info = app.network_info.as_ref().unwrap();
    assert_eq!(info.subnet, "10.0.0.0/16");
}

#[test]
fn submit_edit_server_form() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.form_fields = vec![
        TextInput::with_value("5.6.7.8"),
        TextInput::with_value("12345"),
    ];
    app.form_labels = vec!["Public IP", "Port"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::EditServer {
        name: "srv1".to_string(),
    };

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::Normal));
    assert_eq!(app.servers[0].public_ip, "5.6.7.8");
    assert_eq!(app.servers[0].listen_port, 12345);
}

#[test]
fn submit_edit_client_form() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.form_fields = vec![TextInput::with_value("new_name")];
    app.form_labels = vec!["New Name"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::EditClient {
        name: "client1".to_string(),
    };

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::Normal));
    assert_eq!(app.clients[0].name, "new_name");
}

// ── Form submission: duplicate name error handling ───────────────────

#[test]
fn submit_add_server_duplicate_name_stays_in_form() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.form_fields = vec![
        TextInput::with_value("srv1"), // duplicate
        TextInput::with_value("5.6.7.8"),
        TextInput::with_value("51821"),
    ];
    app.form_labels = vec!["Name", "Public IP", "Port"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::AddServer;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::AddServer));
    assert!(app.status.contains("Error"));
}

#[test]
fn submit_add_client_duplicate_name_stays_in_form() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.form_fields = vec![TextInput::with_value("client1")]; // duplicate
    app.form_labels = vec!["Name"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::AddClient;

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::AddClient));
    assert!(app.status.contains("Error"));
}

#[test]
fn submit_edit_client_to_existing_name_stays_in_form() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.form_fields = vec![TextInput::with_value("srv1")]; // conflicts with server
    app.form_labels = vec!["New Name"];
    app.form_focus = FormFocus::Submit;
    app.mode = Mode::EditClient {
        name: "client1".to_string(),
    };

    app.handle_key(key(KeyCode::Enter));
    assert!(matches!(app.mode, Mode::EditClient { .. }));
    assert!(app.status.contains("Error"));
}
