//! Aggregated test modules for the TUI application.

pub(super) use super::*;
pub(super) use crate::manager::{Message, MessageKind};
pub(super) use crate::tests::*;
pub(super) use crate::tui::input::TextInput;
pub(super) use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(super) fn setup_app() -> (TempDir, App) {
    let (tmp, manager) = setup_empty_manager();
    let app = App {
        manager,
        panel: Panel::OverallInfo,
        mode: Mode::Normal,
        should_quit: false,
        servers: Vec::new(),
        clients: Vec::new(),
        network_info: None,
        device_index: 0,
        form_fields: Vec::new(),
        form_labels: Vec::new(),
        form_focus: FormFocus::Field(0),
        status: String::new(),
        force_init: false,
        export_dir: TextInput::with_value("output"),
        editing_export_dir: false,
        export_dir_backup: String::from("output"),
        selected_device_info: None,
        selected_config: None,
        selected_device_name: None,
        detail_scroll: 0,
        config_scroll: 0,
        logs: Vec::new(),
    };
    (tmp, app)
}

pub(super) fn setup_app_with_network() -> (TempDir, App) {
    let (tmp, mut app) = setup_app();
    app.manager
        .init_network(TEST_SUBNET, TEST_SERVER_START, TEST_SERVER_END, false)
        .unwrap();
    app.refresh_data();
    (tmp, app)
}

pub(super) fn setup_app_with_devices() -> (TempDir, App) {
    let (tmp, mut app) = setup_app_with_network();
    app.manager.add_server("srv1", "1.2.3.4", 51820).unwrap();
    app.manager.add_client("client1").unwrap();
    app.refresh_data();
    (tmp, app)
}

pub(super) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

pub(super) fn key_char(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

pub(super) fn key_ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

mod forms;
mod helpers;
mod key_handling;
mod state;
