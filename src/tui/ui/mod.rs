//! Top-level draw function, layout composition, title bar, and status bar.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::tui::app::{App, Mode};

use super::theme;

mod modals;
mod panels;

// ── Main draw entry point ───────────────────────────────────────

/// Render the full TUI layout: title bar, four quadrants, log panel, status bar, and modal overlays.
pub fn draw(f: &mut Frame, app: &App) {
    // Fill entire terminal with base background
    f.render_widget(
        Block::default().style(Style::default().bg(theme::BG_DARK)),
        f.area(),
    );

    let chunks = Layout::vertical([
        Constraint::Length(1), // title bar
        Constraint::Min(0),    // main content (four quadrants)
        Constraint::Length(6), // log panel
        Constraint::Length(1), // status bar
    ])
    .split(f.area());

    let fallback = f.area();
    let title_area = chunks.first().copied().unwrap_or(fallback);
    let content_area = chunks.get(1).copied().unwrap_or(fallback);
    let log_area = chunks.get(2).copied().unwrap_or(fallback);
    let status_area = chunks.get(3).copied().unwrap_or(fallback);

    draw_title_bar(f, title_area);
    draw_quadrants(f, app, content_area);
    panels::draw_log_panel(f, app, log_area);
    draw_status_bar(f, status_area);

    // Modal overlays
    match &app.mode {
        Mode::InitNetwork
        | Mode::AddServer
        | Mode::AddClient
        | Mode::EditServer { .. }
        | Mode::EditClient { .. }
        | Mode::EditNetworkConfig => {
            modals::draw_form_modal(f, app);
        }
        Mode::ConfirmDelete { name, device_type } => {
            modals::draw_confirm_dialog(f, name, device_type);
        }
        Mode::ConfirmReinit => {
            modals::draw_confirm_reinit(f);
        }
        Mode::ChooseDeviceType => {
            modals::draw_choose_device_type(f);
        }
        Mode::Help => {
            modals::draw_help_overlay(f);
        }
        Mode::Normal => {}
    }
}

// ── Title bar ───────────────────────────────────────────────────

fn draw_title_bar(f: &mut Frame, area: Rect) {
    let bg = Style::default().bg(theme::BG_SURFACE);
    f.render_widget(Block::default().style(bg), area);

    let line = Line::from(vec![Span::styled(
        " WireGuard Manager ",
        Style::default()
            .fg(theme::ACCENT_TEAL)
            .add_modifier(Modifier::BOLD),
    )]);
    f.render_widget(Paragraph::new(line).style(bg), area);
}

// ── Four quadrants ──────────────────────────────────────────────

fn draw_quadrants(f: &mut Frame, app: &App, area: Rect) {
    // Split into left (50%) and right (50%)
    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let left_col = columns.first().copied().unwrap_or(area);
    let right_col = columns.get(1).copied().unwrap_or(area);

    // Left column: top-left (OverallInfo) + bottom-left (DeviceList fills rest)
    let left_rows = Layout::vertical([Constraint::Length(10), Constraint::Min(0)]).split(left_col);
    let top_left = left_rows.first().copied().unwrap_or(left_col);
    let bottom_left = left_rows.get(1).copied().unwrap_or(left_col);

    // Right column: top-right (DeviceDetail) + bottom-right (ConfigPreview fills rest)
    let right_rows =
        Layout::vertical([Constraint::Length(10), Constraint::Min(0)]).split(right_col);
    let top_right = right_rows.first().copied().unwrap_or(right_col);
    let bottom_right = right_rows.get(1).copied().unwrap_or(right_col);

    panels::draw_overall_info(f, app, top_left);
    panels::draw_device_list(f, app, bottom_left);
    panels::draw_device_detail(f, app, top_right);
    panels::draw_config_preview(f, app, bottom_right);
}

// ── Status bar ──────────────────────────────────────────────────

fn draw_status_bar(f: &mut Frame, area: Rect) {
    let bg = Style::default().bg(theme::BG_SURFACE);
    f.render_widget(Block::default().style(bg), area);

    let available = usize::from(area.width);

    // Right segment: shortcuts hint
    let hint = "Q:Quit  ?:Help  TAB:Panel";
    let hint_len = hint.len();

    // Left segment: green checkmark + Ready
    let icon = "\u{2714} ";
    let label = "Ready";
    let left_len = icon.len() + label.len();

    let padding = available.saturating_sub(left_len).saturating_sub(hint_len);

    let line = Line::from(vec![
        Span::styled(icon, Style::default().fg(theme::ACCENT_GREEN)),
        Span::styled(label, Style::default().fg(theme::TEXT_PRIMARY)),
        Span::raw(" ".repeat(padding)),
        Span::styled(hint, Style::default().fg(theme::TEXT_MUTED)),
    ]);
    f.render_widget(Paragraph::new(line).style(bg), area);
}
