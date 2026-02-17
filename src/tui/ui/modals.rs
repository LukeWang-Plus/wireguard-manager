//! Modal overlay rendering: form dialogs, confirmations, device chooser, and help.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

use crate::tui::app::{App, FormFocus, Mode};
use crate::tui::theme;

// ── Form modal ──────────────────────────────────────────────────

pub(super) fn draw_form_modal(f: &mut Frame, app: &App) {
    let title = match &app.mode {
        Mode::InitNetwork => "Initialize Network",
        Mode::AddServer => "Add Server",
        Mode::AddClient => "Add Client",
        Mode::EditServer { name } => {
            return draw_form_modal_titled(f, app, &format!("Edit Server: {name}"));
        }
        Mode::EditClient { name } => {
            return draw_form_modal_titled(f, app, &format!("Edit Client: {name}"));
        }
        Mode::EditNetworkConfig => "Edit Network Config",
        _ => return,
    };
    draw_form_modal_titled(f, app, title);
}

fn draw_form_modal_titled(f: &mut Frame, app: &App, title: &str) {
    let field_count = app.form_fields.len();
    // Each field: 1 label + 3 bordered input = 4 lines;
    // plus 1 blank + 1 buttons + 2 block borders + 2 block padding (top 1 + bottom 1) = +6
    let height = u16::try_from(field_count)
        .unwrap_or(u16::MAX)
        .saturating_mul(4)
        .saturating_add(6);
    let width = 56.min(f.area().width.saturating_sub(4));
    let area = centered_rect(width, height, f.area());

    // Shadow -> Clear -> Content
    theme::render_shadow(f, area);
    f.render_widget(Clear, area);

    let block = theme::focused_block(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut constraints: Vec<Constraint> = Vec::with_capacity(field_count * 2 + 2);
    for _ in 0..field_count {
        constraints.push(Constraint::Length(1)); // label
        constraints.push(Constraint::Length(3)); // bordered input (border top + value + border bottom)
    }
    constraints.push(Constraint::Length(1)); // blank
    constraints.push(Constraint::Length(1)); // buttons

    let chunks = Layout::vertical(constraints).split(inner);

    for i in 0..field_count {
        let Some(label_text) = app.form_labels.get(i) else {
            continue;
        };
        let Some(field) = app.form_fields.get(i) else {
            continue;
        };

        let Some(&label_area) = chunks.get(i * 2) else {
            continue;
        };
        let Some(&input_area) = chunks.get(i * 2 + 1) else {
            continue;
        };

        let is_focused = app.form_focus == FormFocus::Field(i);

        let label_style = if is_focused {
            Style::default()
                .fg(theme::ACCENT_BLUE)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_SECONDARY)
        };
        let label = Paragraph::new(format!("{label_text}:")).style(label_style);
        f.render_widget(label, label_area);

        let (border_color, input_bg) = if is_focused {
            (theme::BORDER_FOCUS, theme::BG_SURFACE)
        } else {
            (theme::BORDER_DIM, theme::BG_DARK)
        };

        let value = field.value();
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(input_bg));
        let input_inner = input_block.inner(input_area);
        f.render_widget(input_block, input_area);

        let input = Paragraph::new(Span::styled(
            value,
            Style::default().fg(theme::TEXT_PRIMARY),
        ));
        f.render_widget(input, input_inner);

        if is_focused {
            let cursor_offset = u16::try_from(field.cursor()).unwrap_or(u16::MAX);
            let cursor_x = input_inner.x.saturating_add(cursor_offset);
            let cursor_y = input_inner.y;
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }

    // Buttons
    let btn_area = chunks.last().copied().unwrap_or(inner);
    draw_form_buttons(f, btn_area, &app.form_focus);
}

fn draw_form_buttons(f: &mut Frame, btn_area: Rect, form_focus: &FormFocus) {
    let btn_chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(btn_area);

    let submit_style = if *form_focus == FormFocus::Submit {
        Style::default()
            .fg(theme::BG_DARK)
            .bg(theme::ACCENT_GREEN)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::ACCENT_GREEN)
    };
    let cancel_style = if *form_focus == FormFocus::Cancel {
        Style::default()
            .fg(theme::BG_DARK)
            .bg(theme::ACCENT_RED)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::ACCENT_RED)
    };

    let submit_area = btn_chunks.first().copied().unwrap_or(btn_area);
    let cancel_area = btn_chunks.get(1).copied().unwrap_or(btn_area);

    f.render_widget(
        Paragraph::new("\u{2714} Submit")
            .style(submit_style)
            .alignment(Alignment::Center),
        submit_area,
    );
    f.render_widget(
        Paragraph::new("\u{2718} Cancel")
            .style(cancel_style)
            .alignment(Alignment::Center),
        cancel_area,
    );
}

// ── Confirm delete dialog ───────────────────────────────────────

pub(super) fn draw_confirm_dialog(f: &mut Frame, name: &str, device_type: &str) {
    let area = centered_rect(48, 7, f.area());
    theme::render_shadow(f, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT_RED))
        .title(" \u{26A0} Confirm Delete ")
        .title_style(
            Style::default()
                .fg(theme::ACCENT_RED)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme::BG_ELEVATED))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                format!("Delete {device_type} "),
                Style::default().fg(theme::TEXT_PRIMARY),
            ),
            Span::styled(
                format!("'{name}'"),
                Style::default()
                    .fg(theme::ACCENT_RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", Style::default().fg(theme::TEXT_PRIMARY)),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                " [Y] ",
                Style::default()
                    .fg(theme::ACCENT_RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Yes   ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                " [N] ",
                Style::default()
                    .fg(theme::ACCENT_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("No", Style::default().fg(theme::TEXT_SECONDARY)),
        ]),
    ];
    f.render_widget(Paragraph::new(text), inner);
}

pub(super) fn draw_confirm_reinit(f: &mut Frame) {
    let area = centered_rect(52, 8, f.area());
    theme::render_shadow(f, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT_YELLOW))
        .title(" \u{26A0} Confirm Re-initialize ")
        .title_style(
            Style::default()
                .fg(theme::ACCENT_YELLOW)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme::BG_ELEVATED))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "This will reset the network configuration.",
            Style::default().fg(theme::TEXT_PRIMARY),
        )),
        Line::from(Span::styled(
            "All existing devices will be removed!",
            Style::default()
                .fg(theme::ACCENT_RED)
                .add_modifier(Modifier::BOLD),
        )),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                " [Y] ",
                Style::default()
                    .fg(theme::ACCENT_RED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Yes   ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                " [N] ",
                Style::default()
                    .fg(theme::ACCENT_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("No", Style::default().fg(theme::TEXT_SECONDARY)),
        ]),
    ];
    f.render_widget(Paragraph::new(text), inner);
}

// ── Choose device type dialog ───────────────────────────────────

pub(super) fn draw_choose_device_type(f: &mut Frame) {
    let area = centered_rect(42, 7, f.area());
    theme::render_shadow(f, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::ACCENT_BLUE))
        .title(" Add Device ")
        .title_style(
            Style::default()
                .fg(theme::ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme::BG_ELEVATED))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::raw(""),
        Line::from(Span::styled(
            "Choose device type:",
            Style::default().fg(theme::TEXT_PRIMARY),
        )),
        Line::raw(""),
        Line::from(vec![
            Span::styled(
                " [S] ",
                Style::default()
                    .fg(theme::ACCENT_CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Server   ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                " [C] ",
                Style::default()
                    .fg(theme::ACCENT_MAGENTA)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Client   ", Style::default().fg(theme::TEXT_SECONDARY)),
            Span::styled(
                " ESC ",
                Style::default()
                    .fg(theme::TEXT_MUTED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Cancel", Style::default().fg(theme::TEXT_MUTED)),
        ]),
    ];
    f.render_widget(Paragraph::new(text), inner);
}

// ── Help overlay ────────────────────────────────────────────────

pub(super) fn draw_help_overlay(f: &mut Frame) {
    let area = centered_rect(58, 29, f.area());
    theme::render_shadow(f, area);
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER_NORMAL))
        .title(" Help \u{2014} Key Bindings ")
        .title_style(
            Style::default()
                .fg(theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(theme::BG_ELEVATED));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let help_text = help_text_lines();
    f.render_widget(Paragraph::new(help_text), inner);
}

fn help_text_lines<'a>() -> Vec<Line<'a>> {
    let section_style = Style::default()
        .fg(theme::ACCENT_ORANGE)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default()
        .fg(theme::ACCENT_TEAL)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme::TEXT_SECONDARY);

    vec![
        Line::styled(" Navigation", section_style),
        Line::from(vec![
            Span::styled("  TAB / SHIFT+TAB  ", key_style),
            Span::styled("Switch Panel", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  UP/K  DOWN/J     ", key_style),
            Span::styled("Navigate List / Scroll Config", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  HOME / END       ", key_style),
            Span::styled("First / Last Device", desc_style),
        ]),
        Line::raw(""),
        Line::styled(" Network", section_style),
        Line::from(vec![
            Span::styled("  CTRL+I           ", key_style),
            Span::styled("Initialize / Reinitialize Network", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  CTRL+E           ", key_style),
            Span::styled("Edit Network Config", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  CTRL+O           ", key_style),
            Span::styled("Edit Export Directory", desc_style),
        ]),
        Line::raw(""),
        Line::styled(" Device List", section_style),
        Line::from(vec![
            Span::styled("  A                ", key_style),
            Span::styled("Add New Device", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  E                ", key_style),
            Span::styled("Edit Selected Device", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  D                ", key_style),
            Span::styled("Delete Selected Device", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  S                ", key_style),
            Span::styled("Generate Device Config", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  G                ", key_style),
            Span::styled("Generate All Configs", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  PGUP / PGDN      ", key_style),
            Span::styled("Scroll Device Detail", desc_style),
        ]),
        Line::raw(""),
        Line::styled(" Forms / Modals", section_style),
        Line::from(vec![
            Span::styled("  TAB / SHIFT+TAB  ", key_style),
            Span::styled("Next / Previous Field", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ENTER            ", key_style),
            Span::styled("Submit / Next Field", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ESC              ", key_style),
            Span::styled("Cancel / Close", desc_style),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("  ?                ", key_style),
            Span::styled("Help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Q                ", key_style),
            Span::styled("Quit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  CTRL+C           ", key_style),
            Span::styled("Force Quit", desc_style),
        ]),
    ]
}

// ── Helpers ─────────────────────────────────────────────────────

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x.saturating_add(area.width.saturating_sub(width) / 2);
    let y = area
        .y
        .saturating_add(area.height.saturating_sub(height) / 2);
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
