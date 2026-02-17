//! Tests for `wrapped_line_count` and `char_chunks` pure logic functions.

use ratatui::text::{Line, Text};

use super::*;

// ── wrapped_line_count ───────────────────────────────────────────

#[test]
fn wrapped_short_line_no_wrap() {
    let text = Text::from("hello");
    assert_eq!(wrapped_line_count(&text, 20), 1);
}

#[test]
fn wrapped_exact_width() {
    let text = Text::from("abcde");
    assert_eq!(wrapped_line_count(&text, 5), 1);
}

#[test]
fn wrapped_needs_wrapping() {
    let text = Text::from("abcdefghij");
    // 10 chars at width 4: ceil(10/4) = 3
    assert_eq!(wrapped_line_count(&text, 4), 3);
}

#[test]
fn wrapped_empty_line_counts_as_one() {
    let text = Text::from("");
    assert_eq!(wrapped_line_count(&text, 10), 1);
}

#[test]
fn wrapped_zero_width_falls_back_to_line_count() {
    let text = Text::from(vec![Line::from("abc"), Line::from("def")]);
    assert_eq!(wrapped_line_count(&text, 0), 2);
}

#[test]
fn wrapped_multi_line_mixed() {
    let text = Text::from(vec![
        Line::from("short"),      // 5 chars in width 3 → ceil(5/3) = 2
        Line::from(""),           // empty → 1
        Line::from("abcdefghij"), // 10 chars in width 3 → ceil(10/3) = 4
    ]);
    assert_eq!(wrapped_line_count(&text, 3), 7);
}

// ── char_chunks ──────────────────────────────────────────────────

#[test]
fn char_chunks_shorter_than_width() {
    let chunks: Vec<&str> = char_chunks("abc", 10).collect();
    assert_eq!(chunks, vec!["abc"]);
}

#[test]
fn char_chunks_exact_width() {
    let chunks: Vec<&str> = char_chunks("abcde", 5).collect();
    assert_eq!(chunks, vec!["abcde"]);
}

#[test]
fn char_chunks_needs_splitting() {
    let chunks: Vec<&str> = char_chunks("abcdefgh", 3).collect();
    assert_eq!(chunks, vec!["abc", "def", "gh"]);
}

#[test]
fn char_chunks_empty_string() {
    let chunks: Vec<&str> = char_chunks("", 5).collect();
    assert!(chunks.is_empty());
}
