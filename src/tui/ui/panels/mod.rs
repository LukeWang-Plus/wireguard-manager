//! Panel rendering: network overview, device list, device detail, config preview, and log.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Cell, Gauge, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState, Wrap,
    },
};

use crate::manager::DeviceInfo;
use crate::tui::app::{App, Panel};
use crate::tui::theme;

/// Compute the total visual line count after wrapping text to the given width.
fn wrapped_line_count(text: &Text, visible_width: u16) -> u16 {
    if visible_width == 0 {
        return u16::try_from(text.lines.len()).unwrap_or(u16::MAX);
    }
    let width = usize::from(visible_width);
    text.lines
        .iter()
        .map(|line| {
            let w = line.width();
            if w == 0 {
                1u16
            } else {
                u16::try_from(w.div_ceil(width)).unwrap_or(u16::MAX)
            }
        })
        .fold(0u16, u16::saturating_add)
}

// ── Top-left: Overall Info ──────────────────────────────────────

pub(super) fn draw_overall_info(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::OverallInfo;
    let block = if focused {
        theme::focused_block("Network Overview")
    } else {
        theme::panel_block("Network Overview")
    };
    let block = match (&app.network_info, app.editing_export_dir) {
        (Some(_), true) => block.title_bottom(
            theme::hint_line(&[("ENTER", "Confirm"), ("ESC", "Cancel"), ("CTRL+U", "Clear")])
                .alignment(Alignment::Center),
        ),
        (Some(_), false) => block.title_bottom(
            theme::hint_line(&[
                ("CTRL+I", "Init"),
                ("CTRL+E", "Edit Network"),
                ("CTRL+O", "Edit Export Dir"),
            ])
            .alignment(Alignment::Center),
        ),
        (None, _) => block,
    };
    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(info) = &app.network_info {
        draw_initialized_overview(f, inner, info, app);
    } else {
        let text = vec![
            Line::from(vec![
                Span::styled(
                    "\u{26A0} ",
                    Style::default()
                        .fg(theme::ACCENT_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Network not initialized",
                    Style::default()
                        .fg(theme::ACCENT_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(theme::TEXT_SECONDARY)),
                Span::styled(
                    "[CTRL+I]",
                    Style::default()
                        .fg(theme::ACCENT_YELLOW)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " to initialize the network",
                    Style::default().fg(theme::TEXT_SECONDARY),
                ),
            ]),
        ];
        f.render_widget(Paragraph::new(text), inner);
    }
}

fn draw_initialized_overview(
    f: &mut Frame,
    inner: Rect,
    info: &crate::manager::NetworkInfo,
    app: &App,
) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // subnet, server range, client range
        Constraint::Length(1), // blank
        Constraint::Length(1), // server label + gauge
        Constraint::Length(1), // client label + gauge
        Constraint::Length(1), // blank
        Constraint::Length(1), // export dir label + input (same line)
    ])
    .split(inner);

    let info_area = chunks.first().copied().unwrap_or(inner);
    let server_gauge_area = chunks.get(2).copied().unwrap_or(inner);
    let client_gauge_area = chunks.get(3).copied().unwrap_or(inner);
    let export_area = chunks.get(5).copied().unwrap_or(inner);

    let net_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Subnet:       ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                &info.subnet,
                Style::default()
                    .fg(theme::TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Server range: ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                format!("{} - {}", info.server_start_ip, info.server_end_ip),
                Style::default().fg(theme::TEXT_PRIMARY),
            ),
        ]),
        Line::from(vec![
            Span::styled("Client range: ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                format!("{} - {}", info.client_start_ip, info.client_end_ip),
                Style::default().fg(theme::TEXT_PRIMARY),
            ),
        ]),
    ]);
    f.render_widget(Paragraph::new(net_text), info_area);

    draw_capacity_gauge(
        f,
        server_gauge_area,
        "Servers:  ",
        info.server_count,
        info.server_capacity,
    );
    draw_capacity_gauge(
        f,
        client_gauge_area,
        "Clients:  ",
        info.client_count,
        info.client_capacity,
    );

    draw_export_dir(f, export_area, app);
}

fn draw_capacity_gauge(f: &mut Frame, area: Rect, label: &str, count: u32, capacity: u32) {
    let label_width: u16 = 10;
    let cols =
        Layout::horizontal([Constraint::Length(label_width), Constraint::Min(0)]).split(area);
    let label_area = cols.first().copied().unwrap_or(area);
    let bar_area = cols.get(1).copied().unwrap_or(area);

    f.render_widget(
        Paragraph::new(Span::styled(
            label,
            Style::default().fg(theme::TEXT_SECONDARY),
        )),
        label_area,
    );

    let ratio = if capacity > 0 {
        f64::from(count) / f64::from(capacity)
    } else {
        0.0
    };
    let free = capacity.saturating_sub(count);
    let gauge = Gauge::default()
        .gauge_style(theme::gauge_color(ratio))
        .style(Style::default().bg(theme::BG_SURFACE))
        .use_unicode(true)
        .ratio(ratio.min(1.0))
        .label(Span::styled(
            format!("{count}/{capacity} ({free} free)"),
            Style::default().fg(theme::TEXT_PRIMARY),
        ));
    f.render_widget(gauge, bar_area);
}

fn draw_export_dir(f: &mut Frame, area: Rect, app: &App) {
    let export_label = "Export Dir: ";
    let export_label_len = u16::try_from(export_label.len()).unwrap_or(12);
    let export_cols =
        Layout::horizontal([Constraint::Length(export_label_len), Constraint::Min(0)]).split(area);
    let export_label_area = export_cols.first().copied().unwrap_or(area);
    let export_input_area = export_cols.get(1).copied().unwrap_or(area);

    let editing = app.editing_export_dir;

    let export_label_style = if editing {
        Style::default()
            .fg(theme::ACCENT_BLUE)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::TEXT_SECONDARY)
    };
    f.render_widget(
        Paragraph::new(export_label).style(export_label_style),
        export_label_area,
    );

    let input_bg = if editing {
        theme::BG_SURFACE
    } else {
        theme::BG_ELEVATED
    };
    let value = app.export_dir.value();
    let input = Paragraph::new(Span::styled(
        value,
        Style::default().fg(theme::TEXT_PRIMARY),
    ))
    .style(Style::default().bg(input_bg));
    f.render_widget(input, export_input_area);

    if editing {
        let cursor_offset = u16::try_from(app.export_dir.cursor()).unwrap_or(u16::MAX);
        let cursor_x = export_input_area.x.saturating_add(cursor_offset);
        let cursor_y = export_input_area.y;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

// ── Bottom-left: Device List ────────────────────────────────────

pub(super) fn draw_device_list(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::DeviceList;
    let block = if focused {
        theme::focused_block("Device List")
    } else {
        theme::panel_block("Device List")
    }
    .title_bottom(
        theme::hint_line(&[
            ("A", "Add"),
            ("E", "Edit"),
            ("D", "Del"),
            ("S", "Generate"),
            ("G", "Generate All"),
        ])
        .alignment(Alignment::Center),
    );
    let inner = block.inner(area);
    f.render_widget(block, area);

    let total = app.servers.len() + app.clients.len();
    if total == 0 {
        let msg = Paragraph::new("No devices. Press [A] to add one")
            .style(Style::default().fg(theme::TEXT_MUTED));
        f.render_widget(msg, inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Type"),
        Cell::from("Name"),
        Cell::from("IP"),
    ])
    .style(theme::table_header_style())
    .bottom_margin(1);

    let mut rows: Vec<Row> = Vec::with_capacity(total);
    let server_rows = app.servers.iter().map(|s| {
        (
            Cell::from("Server").style(Style::default().fg(theme::ACCENT_CYAN)),
            s.name.as_str(),
            s.ip.as_str(),
        )
    });
    let client_rows = app.clients.iter().map(|c| {
        (
            Cell::from("Client").style(Style::default().fg(theme::ACCENT_MAGENTA)),
            c.name.as_str(),
            c.ip.as_str(),
        )
    });
    for (row_idx, (type_cell, name, ip)) in server_rows.chain(client_rows).enumerate() {
        let bg = if row_idx % 2 == 1 {
            theme::TABLE_ROW_ALT
        } else {
            theme::BG_MID
        };
        rows.push(
            Row::new(vec![type_cell, Cell::from(name), Cell::from(ip)])
                .style(Style::default().bg(bg).fg(theme::TEXT_PRIMARY)),
        );
    }

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(40),
        Constraint::Percentage(40),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::table_highlight_style())
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_symbol("\u{25B6} ")
        .column_spacing(2);

    let mut state = TableState::default();
    state.select(Some(app.device_index));
    f.render_stateful_widget(table, inner, &mut state);
}

// ── Top-right: Device Detail ────────────────────────────────────

pub(super) fn draw_device_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = theme::panel_block("Device Detail");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(ref info) = app.selected_device_info else {
        let msg = Paragraph::new("Select a device to view details")
            .style(Style::default().fg(theme::TEXT_MUTED));
        f.render_widget(msg, inner);
        return;
    };

    let lines = device_detail_lines(info);
    let text = Text::from(lines);
    let line_count = wrapped_line_count(&text, inner.width);
    let visible = inner.height;
    let max_scroll = line_count.saturating_sub(visible);
    let scroll = app.detail_scroll.min(max_scroll);

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(paragraph, inner);

    if line_count > visible {
        render_scrollbar(f, inner, max_scroll, scroll);
    }
}

fn device_detail_lines(info: &DeviceInfo) -> Vec<Line<'_>> {
    match info {
        DeviceInfo::Server {
            name,
            ip,
            public_ip,
            listen_port,
            private_key,
            public_key,
            preshared_keys_count,
        } => {
            vec![
                Line::from(vec![
                    Span::styled(
                        "\u{25A0} Server: ",
                        Style::default()
                            .fg(theme::ACCENT_CYAN)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        name,
                        Style::default()
                            .fg(theme::TEXT_PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Internal IP:  ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(ip, Style::default().fg(theme::TEXT_PRIMARY)),
                ]),
                Line::from(vec![
                    Span::styled("Public IP:    ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(public_ip, Style::default().fg(theme::TEXT_PRIMARY)),
                ]),
                Line::from(vec![
                    Span::styled("Port:         ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(
                        listen_port.to_string(),
                        Style::default().fg(theme::TEXT_PRIMARY),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Private Key:  ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(private_key, Style::default().fg(theme::TEXT_PRIMARY)),
                ]),
                Line::from(vec![
                    Span::styled("Public Key:   ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(public_key, Style::default().fg(theme::TEXT_PRIMARY)),
                ]),
                Line::from(vec![
                    Span::styled("PSK Count:    ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(
                        preshared_keys_count.to_string(),
                        Style::default().fg(theme::TEXT_PRIMARY),
                    ),
                ]),
            ]
        }
        DeviceInfo::Client {
            name,
            ip,
            private_key,
            public_key,
        } => {
            vec![
                Line::from(vec![
                    Span::styled(
                        "\u{25CF} Client: ",
                        Style::default()
                            .fg(theme::ACCENT_MAGENTA)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        name,
                        Style::default()
                            .fg(theme::TEXT_PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Internal IP:  ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(ip, Style::default().fg(theme::TEXT_PRIMARY)),
                ]),
                Line::from(vec![
                    Span::styled("Private Key:  ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(private_key, Style::default().fg(theme::TEXT_PRIMARY)),
                ]),
                Line::from(vec![
                    Span::styled("Public Key:   ", Style::default().fg(theme::TEXT_SECONDARY)),
                    Span::styled(public_key, Style::default().fg(theme::TEXT_PRIMARY)),
                ]),
            ]
        }
    }
}

fn render_scrollbar(f: &mut Frame, inner: Rect, max_scroll: u16, scroll: u16) {
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(theme::BORDER_NORMAL))
        .track_style(Style::default().fg(theme::BORDER_DIM));
    let mut scrollbar_state =
        ScrollbarState::new(usize::from(max_scroll)).position(usize::from(scroll));
    f.render_stateful_widget(
        scrollbar,
        inner.inner(Margin {
            vertical: 0,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

// ── Bottom-right: Config Preview ────────────────────────────────

pub(super) fn draw_config_preview(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.panel == Panel::ConfigPreview;
    let title = app.selected_device_name.as_ref().map_or_else(
        || "Config Preview".to_string(),
        |name| format!("Config: {name}.conf"),
    );
    let block = if focused {
        theme::focused_block(&title)
    } else {
        theme::panel_block(&title)
    };
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(ref content) = app.selected_config else {
        let msg = Paragraph::new("Select a device to preview config")
            .style(Style::default().fg(theme::TEXT_MUTED));
        f.render_widget(msg, inner);
        return;
    };

    // Simple syntax highlighting for WireGuard config
    let highlighted_lines: Vec<Line> = content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(theme::ACCENT_ORANGE)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if let Some(eq_pos) = line.find('=') {
                let (key_part, rest) = line.split_at(eq_pos);
                let (eq_sign, val_part) = rest.split_at(1);
                Line::from(vec![
                    Span::styled(key_part, Style::default().fg(theme::ACCENT_TEAL)),
                    Span::styled(eq_sign, Style::default().fg(theme::TEXT_MUTED)),
                    Span::styled(val_part, Style::default().fg(theme::TEXT_PRIMARY)),
                ])
            } else {
                Line::from(Span::styled(line, Style::default().fg(theme::TEXT_PRIMARY)))
            }
        })
        .collect();

    let text = Text::from(highlighted_lines);
    let line_count = wrapped_line_count(&text, inner.width);
    let visible = inner.height;
    let max_scroll = line_count.saturating_sub(visible);
    let scroll = app.config_scroll.min(max_scroll);

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(paragraph, inner);

    if line_count > visible {
        render_scrollbar(f, inner, max_scroll, scroll);
    }
}

// ── Bottom: Log Panel ───────────────────────────────────────────

pub(super) fn draw_log_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = theme::panel_block("Log");
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.logs.is_empty() {
        let msg = Paragraph::new("No logs yet").style(Style::default().fg(theme::TEXT_MUTED));
        f.render_widget(msg, inner);
        return;
    }

    // Timestamp prefix width: "[2026-02-16 12:34:56] " = 22 chars
    let ts_width: usize = 22;
    let content_width = usize::from(inner.width).saturating_sub(ts_width);
    let visible_lines = usize::from(inner.height);

    // Build wrapped lines from log entries (newest at bottom)
    let mut all_lines: Vec<Line> = Vec::new();
    for (ts, msg) in &app.logs {
        if content_width == 0 || msg.is_empty() {
            all_lines.push(Line::from(vec![
                Span::styled(format!("{ts} "), Style::default().fg(theme::TEXT_MUTED)),
                Span::styled(msg.as_str(), Style::default().fg(theme::TEXT_PRIMARY)),
            ]));
            continue;
        }

        let mut first = true;
        for chunk in char_chunks(msg, content_width) {
            if first {
                all_lines.push(Line::from(vec![
                    Span::styled(format!("{ts} "), Style::default().fg(theme::TEXT_MUTED)),
                    Span::styled(chunk.to_owned(), Style::default().fg(theme::TEXT_PRIMARY)),
                ]));
                first = false;
            } else {
                all_lines.push(Line::from(vec![
                    Span::raw(" ".repeat(ts_width)),
                    Span::styled(chunk.to_owned(), Style::default().fg(theme::TEXT_PRIMARY)),
                ]));
            }
        }
    }

    // Show only the last N lines (auto-scroll to bottom)
    let skip = all_lines.len().saturating_sub(visible_lines);
    let text = Text::from(all_lines[skip..].to_vec());
    f.render_widget(Paragraph::new(text), inner);
}

/// Split a string into chunks of at most `width` characters, respecting char boundaries.
fn char_chunks(s: &str, width: usize) -> impl Iterator<Item = &str> {
    let mut remaining = s;
    std::iter::from_fn(move || {
        if remaining.is_empty() {
            return None;
        }
        let end = remaining
            .char_indices()
            .nth(width)
            .map_or(remaining.len(), |(idx, _)| idx);
        let chunk = &remaining[..end];
        remaining = &remaining[end..];
        Some(chunk)
    })
}

#[cfg(test)]
mod tests;
