//! Centralized color palette, block builders, style factories, and shadow rendering.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding},
};

// ── Background layers ────────────────────────────────────────────────────

pub const BG_DARK: Color = Color::Rgb(22, 24, 33);
pub const BG_MID: Color = Color::Rgb(30, 33, 46);
pub const BG_ELEVATED: Color = Color::Rgb(40, 44, 62);
pub const BG_SURFACE: Color = Color::Rgb(52, 57, 78);

// ── Text layers ──────────────────────────────────────────────────────────

pub const TEXT_PRIMARY: Color = Color::Rgb(220, 224, 240);
pub const TEXT_SECONDARY: Color = Color::Rgb(150, 156, 180);
pub const TEXT_MUTED: Color = Color::Rgb(95, 100, 126);

// ── Accent colors ────────────────────────────────────────────────────────

pub const ACCENT_BLUE: Color = Color::Rgb(100, 160, 255);
pub const ACCENT_CYAN: Color = Color::Rgb(86, 210, 220);
pub const ACCENT_GREEN: Color = Color::Rgb(80, 210, 130);
pub const ACCENT_YELLOW: Color = Color::Rgb(240, 200, 80);
pub const ACCENT_RED: Color = Color::Rgb(240, 96, 110);
pub const ACCENT_MAGENTA: Color = Color::Rgb(190, 130, 255);
pub const ACCENT_ORANGE: Color = Color::Rgb(245, 150, 80);
pub const ACCENT_TEAL: Color = Color::Rgb(70, 190, 170);

// ── Border colors ────────────────────────────────────────────────────────

pub const BORDER_DIM: Color = Color::Rgb(55, 60, 82);
pub const BORDER_NORMAL: Color = Color::Rgb(75, 82, 110);
pub const BORDER_FOCUS: Color = Color::Rgb(100, 160, 255); // == ACCENT_BLUE

// ── Table colors ─────────────────────────────────────────────────────────

pub const TABLE_ROW_ALT: Color = Color::Rgb(27, 30, 42);
pub const TABLE_HIGHLIGHT: Color = Color::Rgb(45, 60, 100);
pub const TABLE_HEADER_BG: Color = Color::Rgb(35, 40, 58);

// ── Gauge / Shadow ───────────────────────────────────────────────────────

pub const SHADOW: Color = Color::Rgb(10, 10, 16);

// ── Block builders ───────────────────────────────────────────────────────

/// Standard panel block: rounded border, `BORDER_DIM` color, `BG_MID` background.
pub fn panel_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_DIM))
        .title(format!(" {title} "))
        .title_style(
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(BG_MID))
        .padding(Padding::horizontal(1))
}

/// Focused panel block: rounded border, `BORDER_FOCUS` (blue) color, `BG_ELEVATED` background.
pub fn focused_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_FOCUS))
        .title(format!(" {title} "))
        .title_style(
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(BG_ELEVATED))
        .padding(Padding::horizontal(1))
}

/// Table header style: teal bold on dark header background.
pub fn table_header_style() -> Style {
    Style::default()
        .fg(ACCENT_TEAL)
        .bg(TABLE_HEADER_BG)
        .add_modifier(Modifier::BOLD)
}

/// Selected-row highlight style: blue background + bold.
pub fn table_highlight_style() -> Style {
    Style::default()
        .bg(TABLE_HIGHLIGHT)
        .fg(TEXT_PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Dynamic gauge color based on usage ratio.
pub fn gauge_color(ratio: f64) -> Style {
    if ratio < 0.6 {
        Style::default().fg(ACCENT_GREEN)
    } else if ratio < 0.85 {
        Style::default().fg(ACCENT_YELLOW)
    } else {
        Style::default().fg(ACCENT_RED)
    }
}

/// Render a dark shadow rectangle at `(area.x+2, area.y+1)`, clipped to terminal bounds.
pub fn render_shadow(f: &mut Frame, area: Rect) {
    let term = f.area();
    let shadow_x = area.x.saturating_add(2);
    let shadow_y = area.y.saturating_add(1);

    if shadow_x >= term.width || shadow_y >= term.height {
        return;
    }

    let shadow_w = area.width.min(term.width.saturating_sub(shadow_x));
    let shadow_h = area.height.min(term.height.saturating_sub(shadow_y));

    if shadow_w == 0 || shadow_h == 0 {
        return;
    }

    let shadow_rect = Rect::new(shadow_x, shadow_y, shadow_w, shadow_h);
    f.render_widget(Clear, shadow_rect);
    let shadow_block = Block::default().style(Style::default().bg(SHADOW));
    f.render_widget(shadow_block, shadow_rect);
}

/// Build a bottom-bar hint line: keys in `ACCENT_ORANGE` bold, descriptions in `TEXT_MUTED`.
/// `pairs` is a slice of `(key, description)`.
pub fn hint_line<'a>(pairs: &'a [(&'a str, &'a str)]) -> Line<'a> {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(ACCENT_ORANGE)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(":{desc}"),
            Style::default().fg(TEXT_MUTED),
        ));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests;
