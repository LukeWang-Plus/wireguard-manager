//! Tests for helper methods, panel cycling, status/logging, and data refresh.

use super::*;

// ── Panel cycling ────────────────────────────────────────────────────

#[test]
fn panel_next_cycles_through_all_panels() {
    let p = Panel::OverallInfo;
    assert_eq!(p.next(), Panel::DeviceList);
    assert_eq!(p.next().next(), Panel::ConfigPreview);
    assert_eq!(p.next().next().next(), Panel::OverallInfo);
}

#[test]
fn panel_prev_cycles_in_reverse() {
    let p = Panel::OverallInfo;
    assert_eq!(p.prev(), Panel::ConfigPreview);
    assert_eq!(p.prev().prev(), Panel::DeviceList);
    assert_eq!(p.prev().prev().prev(), Panel::OverallInfo);
}

// ── App helper methods ───────────────────────────────────────────────

#[test]
fn device_count_empty() {
    let (_tmp, app) = setup_app();
    assert_eq!(app.device_count(), 0);
}

#[test]
fn device_count_with_devices() {
    let (_tmp, app) = setup_app_with_devices();
    assert_eq!(app.device_count(), 2);
}

#[test]
fn device_name_at_server_index() {
    let (_tmp, app) = setup_app_with_devices();
    assert_eq!(app.device_name_at(0).unwrap(), "srv1");
}

#[test]
fn device_name_at_client_index() {
    let (_tmp, app) = setup_app_with_devices();
    assert_eq!(app.device_name_at(1).unwrap(), "client1");
}

#[test]
fn device_name_at_out_of_bounds() {
    let (_tmp, app) = setup_app_with_devices();
    assert!(app.device_name_at(99).is_none());
}

#[test]
fn is_server_at_true_for_server_index() {
    let (_tmp, app) = setup_app_with_devices();
    assert!(app.is_server_at(0));
}

#[test]
fn is_server_at_false_for_client_index() {
    let (_tmp, app) = setup_app_with_devices();
    assert!(!app.is_server_at(1));
}

#[test]
fn clamp_device_index_within_range() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.device_index = 1;
    app.clamp_device_index();
    assert_eq!(app.device_index, 1);
}

#[test]
fn clamp_device_index_out_of_range() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.device_index = 99;
    app.clamp_device_index();
    assert_eq!(app.device_index, 1); // total=2, clamped to 1
}

#[test]
fn clamp_device_index_empty_resets_to_zero() {
    let (_tmp, mut app) = setup_app();
    app.device_index = 5;
    app.clamp_device_index();
    assert_eq!(app.device_index, 0);
}

// ── Status / logging ─────────────────────────────────────────────────

#[test]
fn set_status_updates_status_field() {
    let (_tmp, mut app) = setup_app();
    app.set_status("test message");
    assert_eq!(app.status, "test message");
}

#[test]
fn set_status_appends_to_logs() {
    let (_tmp, mut app) = setup_app();
    app.set_status("msg1");
    app.set_status("msg2");
    assert_eq!(app.logs.len(), 2);
    assert_eq!(app.logs[0].1, "msg1");
    assert_eq!(app.logs[1].1, "msg2");
}

// ── refresh_data / update_selected_device ────────────────────────────

#[test]
fn refresh_data_populates_caches() {
    let (_tmp, app) = setup_app_with_devices();
    assert!(!app.servers.is_empty());
    assert!(!app.clients.is_empty());
    assert!(app.network_info.is_some());
}

#[test]
fn update_selected_device_resets_scroll() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.detail_scroll = 10;
    app.config_scroll = 10;
    app.update_selected_device();
    assert_eq!(app.detail_scroll, 0);
    assert_eq!(app.config_scroll, 0);
}

#[test]
fn update_selected_device_clears_when_no_devices() {
    let (_tmp, mut app) = setup_app();
    app.update_selected_device();
    assert!(app.selected_device_info.is_none());
    assert!(app.selected_config.is_none());
    assert!(app.selected_device_name.is_none());
}

#[test]
fn update_selected_device_sets_info_for_valid_index() {
    let (_tmp, mut app) = setup_app_with_devices();
    app.device_index = 0;
    app.update_selected_device();
    assert_eq!(app.selected_device_name.as_deref(), Some("srv1"));
    assert!(app.selected_device_info.is_some());
}
