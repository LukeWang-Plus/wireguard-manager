//! Tests for theme color functions, block builders, style factories, and hint line.

use super::*;

// ── gauge_color ──────────────────────────────────────────────────

#[test]
fn gauge_color_zero_is_green() {
    assert_eq!(gauge_color(0.0), Style::default().fg(ACCENT_GREEN));
}

#[test]
fn gauge_color_just_below_medium_threshold() {
    assert_eq!(gauge_color(0.59), Style::default().fg(ACCENT_GREEN));
}

#[test]
fn gauge_color_at_medium_threshold_is_yellow() {
    assert_eq!(gauge_color(0.6), Style::default().fg(ACCENT_YELLOW));
}

#[test]
fn gauge_color_just_below_high_threshold() {
    assert_eq!(gauge_color(0.84), Style::default().fg(ACCENT_YELLOW));
}

#[test]
fn gauge_color_at_high_threshold_is_red() {
    assert_eq!(gauge_color(0.85), Style::default().fg(ACCENT_RED));
}

#[test]
fn gauge_color_full_is_red() {
    assert_eq!(gauge_color(1.0), Style::default().fg(ACCENT_RED));
}

// ── panel_block ──────────────────────────────────────────────────

#[test]
fn panel_block_has_rounded_border_with_dim_color() {
    let block = panel_block("Test");
    let expected = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_DIM))
        .title(" Test ")
        .title_style(
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(BG_MID))
        .padding(Padding::horizontal(1));
    assert_eq!(block, expected);
}

#[test]
fn panel_block_embeds_title_with_spaces() {
    let block = panel_block("Network");
    let expected = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_DIM))
        .title(" Network ")
        .title_style(
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(BG_MID))
        .padding(Padding::horizontal(1));
    assert_eq!(block, expected);
}

// ── focused_block ────────────────────────────────────────────────

#[test]
fn focused_block_uses_focus_border_and_elevated_bg() {
    let block = focused_block("Active");
    let expected = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_FOCUS))
        .title(" Active ")
        .title_style(
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(BG_ELEVATED))
        .padding(Padding::horizontal(1));
    assert_eq!(block, expected);
}

#[test]
fn focused_block_title_uses_accent_blue() {
    let block = focused_block("Panel");
    let expected = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_FOCUS))
        .title(" Panel ")
        .title_style(
            Style::default()
                .fg(ACCENT_BLUE)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().bg(BG_ELEVATED))
        .padding(Padding::horizontal(1));
    assert_eq!(block, expected);
}

// ── table_header_style ───────────────────────────────────────────

#[test]
fn table_header_style_is_teal_bold_on_header_bg() {
    let style = table_header_style();
    let expected = Style::default()
        .fg(ACCENT_TEAL)
        .bg(TABLE_HEADER_BG)
        .add_modifier(Modifier::BOLD);
    assert_eq!(style, expected);
}

#[test]
fn table_header_style_has_bold_modifier() {
    let style = table_header_style();
    assert!(style.add_modifier == Modifier::BOLD);
}

// ── table_highlight_style ────────────────────────────────────────

#[test]
fn table_highlight_style_uses_highlight_bg_and_primary_fg() {
    let style = table_highlight_style();
    let expected = Style::default()
        .bg(TABLE_HIGHLIGHT)
        .fg(TEXT_PRIMARY)
        .add_modifier(Modifier::BOLD);
    assert_eq!(style, expected);
}

#[test]
fn table_highlight_style_is_bold() {
    let style = table_highlight_style();
    assert!(style.add_modifier == Modifier::BOLD);
}

// ── hint_line ────────────────────────────────────────────────────

#[test]
fn hint_line_empty_pairs_returns_empty_line() {
    let line = hint_line(&[]);
    assert!(line.spans.is_empty());
}

#[test]
fn hint_line_single_pair() {
    let line = hint_line(&[("Q", "Quit")]);
    assert_eq!(line.spans.len(), 2); // key + desc
    assert_eq!(line.spans[0].content, "Q");
    assert_eq!(line.spans[1].content, ":Quit");
}

#[test]
fn hint_line_multiple_pairs_have_separators() {
    let line = hint_line(&[("Q", "Quit"), ("?", "Help")]);
    // separator + key + desc for second pair = 5 total
    assert_eq!(line.spans.len(), 5);
    assert_eq!(line.spans[2].content, "  "); // separator
    assert_eq!(line.spans[3].content, "?");
    assert_eq!(line.spans[4].content, ":Help");
}

#[test]
fn hint_line_key_style_is_orange_bold() {
    let line = hint_line(&[("TAB", "Panel")]);
    let key_span = &line.spans[0];
    let expected_style = Style::default()
        .fg(ACCENT_ORANGE)
        .add_modifier(Modifier::BOLD);
    assert_eq!(key_span.style, expected_style);
}
