//! Tests for keyboard event handling across all modes.

use super::*;

// ── Key handling: global ─────────────────────────────────────────────

#[test]
fn ctrl_c_quits_from_any_mode() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::Help;
    app.handle_key(key_ctrl('c'));
    assert!(app.should_quit);
}

#[test]
fn tab_cycles_panel_forward() {
    let (_tmp, mut app) = setup_app();
    assert_eq!(app.panel, Panel::OverallInfo);
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.panel, Panel::DeviceList);
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.panel, Panel::ConfigPreview);
    app.handle_key(key(KeyCode::Tab));
    assert_eq!(app.panel, Panel::OverallInfo);
}

#[test]
fn backtab_cycles_panel_backward() {
    let (_tmp, mut app) = setup_app();
    app.handle_key(key(KeyCode::BackTab));
    assert_eq!(app.panel, Panel::ConfigPreview);
    app.handle_key(key(KeyCode::BackTab));
    assert_eq!(app.panel, Panel::DeviceList);
}

#[test]
fn question_mark_opens_help() {
    let (_tmp, mut app) = setup_app();
    app.handle_key(key_char('?'));
    assert!(matches!(app.mode, Mode::Help));
}

// ── Key handling: help mode ──────────────────────────────────────────

#[test]
fn help_esc_returns_to_normal() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::Help;
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn help_question_mark_returns_to_normal() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::Help;
    app.handle_key(key_char('?'));
    assert!(matches!(app.mode, Mode::Normal));
}

#[test]
fn help_q_returns_to_normal() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::Help;
    app.handle_key(key_char('q'));
    assert!(matches!(app.mode, Mode::Normal));
}

// ── Key handling: choose device type ─────────────────────────────────

#[test]
fn choose_s_opens_add_server_form() {
    let (_tmp, mut app) = setup_app_with_network();
    app.mode = Mode::ChooseDeviceType;
    app.handle_key(key_char('s'));
    assert!(matches!(app.mode, Mode::AddServer));
    assert_eq!(app.form_labels.len(), 3); // Name, Public IP, Port
}

#[test]
fn choose_c_opens_add_client_form() {
    let (_tmp, mut app) = setup_app_with_network();
    app.mode = Mode::ChooseDeviceType;
    app.handle_key(key_char('c'));
    assert!(matches!(app.mode, Mode::AddClient));
    assert_eq!(app.form_labels.len(), 1); // Name
}

#[test]
fn choose_esc_cancels() {
    let (_tmp, mut app) = setup_app();
    app.mode = Mode::ChooseDeviceType;
    app.handle_key(key(KeyCode::Esc));
    assert!(matches!(app.mode, Mode::Normal));
}

// ── Key handling: info panel ─────────────────────────────────────────

#[test]
fn info_panel_q_quits() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.handle_key(key_char('q'));
    assert!(app.should_quit);
}

#[test]
fn info_panel_ctrl_o_starts_export_dir_editing() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.handle_key(key_ctrl('o'));
    assert!(app.editing_export_dir);
    assert_eq!(app.export_dir_backup, "output");
}

#[test]
fn export_dir_esc_restores_backup() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.editing_export_dir = true;
    app.export_dir_backup = "original".to_string();
    app.export_dir = TextInput::with_value("modified");
    app.handle_key(key(KeyCode::Esc));
    assert!(!app.editing_export_dir);
    assert_eq!(app.export_dir.value(), "original");
}

#[test]
fn export_dir_enter_commits() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.editing_export_dir = true;
    app.export_dir = TextInput::with_value("new_dir");
    app.handle_key(key(KeyCode::Enter));
    assert!(!app.editing_export_dir);
    assert_eq!(app.export_dir.value(), "new_dir");
}

#[test]
fn export_dir_enter_rejects_empty() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.editing_export_dir = true;
    app.export_dir = TextInput::with_value("");
    app.handle_key(key(KeyCode::Enter));
    assert!(app.editing_export_dir); // still editing
    assert!(app.status.contains("empty"));
}

#[test]
fn export_dir_char_input() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.editing_export_dir = true;
    app.export_dir = TextInput::new();
    app.handle_key(key_char('x'));
    assert_eq!(app.export_dir.value(), "x");
}

#[test]
fn export_dir_ctrl_u_clears() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.editing_export_dir = true;
    app.export_dir = TextInput::with_value("some_dir");
    app.handle_key(key_ctrl('u'));
    assert_eq!(app.export_dir.value(), "");
}

#[test]
fn tab_while_editing_export_dir_cancels_edit() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::OverallInfo;
    app.editing_export_dir = true;
    app.export_dir_backup = "original".to_string();
    app.export_dir = TextInput::with_value("modified");
    app.handle_key(key(KeyCode::Tab));
    assert!(!app.editing_export_dir);
    assert_eq!(app.export_dir.value(), "original");
    assert_eq!(app.panel, Panel::DeviceList);
}

// ── Key handling: device list ────────────────────────────────────────

#[test]
fn device_list_q_quits() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.handle_key(key_char('q'));
    assert!(app.should_quit);
}

#[test]
fn device_list_down_increments_index() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    assert_eq!(app.device_index, 0);
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.device_index, 1);
}

#[test]
fn device_list_j_increments_index() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.handle_key(key_char('j'));
    assert_eq!(app.device_index, 1);
}

#[test]
fn device_list_up_at_zero_is_noop() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.device_index, 0);
}

#[test]
fn device_list_up_decrements_index() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.device_index = 1;
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.device_index, 0);
}

#[test]
fn device_list_home_goes_to_first() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.device_index = 1;
    app.handle_key(key(KeyCode::Home));
    assert_eq!(app.device_index, 0);
}

#[test]
fn device_list_end_goes_to_last() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.handle_key(key(KeyCode::End));
    assert_eq!(app.device_index, 1);
}

#[test]
fn device_list_a_opens_choose_device_type() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.handle_key(key_char('a'));
    assert!(matches!(app.mode, Mode::ChooseDeviceType));
}

#[test]
fn device_list_pageup_scrolls_detail() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.detail_scroll = 5;
    app.handle_key(key(KeyCode::PageUp));
    assert_eq!(app.detail_scroll, 4);
}

#[test]
fn device_list_pagedown_scrolls_detail() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.handle_key(key(KeyCode::PageDown));
    assert_eq!(app.detail_scroll, 1);
}

// ── Key handling: config preview ─────────────────────────────────────

#[test]
fn config_preview_q_quits() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::ConfigPreview;
    app.handle_key(key_char('q'));
    assert!(app.should_quit);
}

#[test]
fn config_preview_down_scrolls() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::ConfigPreview;
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.config_scroll, 1);
}

#[test]
fn config_preview_up_scrolls() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::ConfigPreview;
    app.config_scroll = 3;
    app.handle_key(key(KeyCode::Up));
    assert_eq!(app.config_scroll, 2);
}

#[test]
fn config_preview_k_scrolls_up() {
    let (_tmp, mut app) = setup_app();
    app.panel = Panel::ConfigPreview;
    app.config_scroll = 3;
    app.handle_key(key_char('k'));
    assert_eq!(app.config_scroll, 2);
}

// ── Key handling: Ctrl+I / Ctrl+E ────────────────────────────────────

#[test]
fn ctrl_i_opens_init_form_when_no_network() {
    let (_tmp, mut app) = setup_app();
    app.handle_key(key_ctrl('i'));
    assert!(matches!(app.mode, Mode::InitNetwork));
    assert!(!app.force_init);
}

#[test]
fn ctrl_i_opens_reinit_confirm_when_network_exists() {
    let (_tmp, mut app) = setup_app_with_network();
    app.handle_key(key_ctrl('i'));
    assert!(matches!(app.mode, Mode::ConfirmReinit));
}

#[test]
fn ctrl_e_opens_edit_network_when_initialized() {
    let (_tmp, mut app) = setup_app_with_network();
    app.handle_key(key_ctrl('e'));
    assert!(matches!(app.mode, Mode::EditNetworkConfig));
    assert_eq!(app.form_labels.len(), 2);
}

#[test]
fn ctrl_e_shows_error_when_not_initialized() {
    let (_tmp, mut app) = setup_app();
    app.handle_key(key_ctrl('e'));
    assert!(matches!(app.mode, Mode::Normal));
    assert!(app.status.contains("not initialized"));
}

// ── Down at last item is noop ────────────────────────────────────────

#[test]
fn device_list_down_at_last_is_noop() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.panel = Panel::DeviceList;
    app.device_index = 1; // last item
    app.handle_key(key(KeyCode::Down));
    assert_eq!(app.device_index, 1);
}

// ── Key handling: s key (generate_and_save_selected) ─────────────────

#[test]
fn s_key_saves_selected_config() {
    let (tmp, mut app) = setup_app_with_devices();
    let out = tmp.path().join("out");
    app.export_dir = crate::tui::input::TextInput::with_value(out.to_str().unwrap());
    app.panel = Panel::DeviceList;
    app.device_index = 0;
    app.handle_key(key_char('s'));
    assert!(app.status.contains("saved") || app.status.contains("Config saved"));
    assert!(out.join("srv1.conf").exists());
}

#[test]
fn s_key_empty_export_dir_shows_error() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.export_dir = crate::tui::input::TextInput::with_value("");
    app.panel = Panel::DeviceList;
    app.device_index = 0;
    app.handle_key(key_char('s'));
    assert!(app.status.contains("empty"));
}

// ── Key handling: g key (generate_all) ───────────────────────────────

#[test]
fn g_key_generates_all_configs() {
    let (tmp, mut app) = setup_app_with_devices();
    let out = tmp.path().join("all_out");
    app.export_dir = crate::tui::input::TextInput::with_value(out.to_str().unwrap());
    app.panel = Panel::DeviceList;
    app.handle_key(key_char('g'));
    assert!(app.status.contains("Generated"));
    assert!(out.join("srv1.conf").exists());
    assert!(out.join("client1.conf").exists());
}

#[test]
fn g_key_empty_export_dir_shows_error() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.export_dir = crate::tui::input::TextInput::with_value("");
    app.panel = Panel::DeviceList;
    app.handle_key(key_char('g'));
    assert!(app.status.contains("empty"));
}
