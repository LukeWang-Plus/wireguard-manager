//! Tests for the `TextInput` widget.

use super::*;

// ── new / with_value ─────────────────────────────────────────────

#[test]
fn new_creates_empty_input() {
    let input = TextInput::new();
    assert_eq!(input.value(), "");
    assert_eq!(input.cursor(), 0);
}

#[test]
fn with_value_sets_content_and_cursor_at_end() {
    let input = TextInput::with_value("hello");
    assert_eq!(input.value(), "hello");
    assert_eq!(input.cursor(), 5);
}

// ── insert ───────────────────────────────────────────────────────

#[test]
fn insert_appends_at_end() {
    let mut input = TextInput::new();
    input.insert('a');
    input.insert('b');
    assert_eq!(input.value(), "ab");
    assert_eq!(input.cursor(), 2);
}

#[test]
fn insert_at_middle() {
    let mut input = TextInput::with_value("ac");
    input.move_left();
    input.insert('b');
    assert_eq!(input.value(), "abc");
    assert_eq!(input.cursor(), 2);
}

#[test]
fn insert_at_beginning() {
    let mut input = TextInput::with_value("bc");
    input.move_home();
    input.insert('a');
    assert_eq!(input.value(), "abc");
    assert_eq!(input.cursor(), 1);
}

// ── delete_back ──────────────────────────────────────────────────

#[test]
fn delete_back_at_start_is_noop() {
    let mut input = TextInput::with_value("abc");
    input.move_home();
    input.delete_back();
    assert_eq!(input.value(), "abc");
    assert_eq!(input.cursor(), 0);
}

#[test]
fn delete_back_removes_preceding_char() {
    let mut input = TextInput::with_value("abc");
    input.delete_back();
    assert_eq!(input.value(), "ab");
    assert_eq!(input.cursor(), 2);
}

#[test]
fn delete_back_in_middle() {
    let mut input = TextInput::with_value("abc");
    input.move_left();
    input.delete_back();
    assert_eq!(input.value(), "ac");
    assert_eq!(input.cursor(), 1);
}

// ── delete_forward ───────────────────────────────────────────────

#[test]
fn delete_forward_at_end_is_noop() {
    let mut input = TextInput::with_value("abc");
    input.delete_forward();
    assert_eq!(input.value(), "abc");
}

#[test]
fn delete_forward_removes_following_char() {
    let mut input = TextInput::with_value("abc");
    input.move_home();
    input.delete_forward();
    assert_eq!(input.value(), "bc");
    assert_eq!(input.cursor(), 0);
}

#[test]
fn delete_forward_in_middle() {
    let mut input = TextInput::with_value("abc");
    input.move_home();
    input.move_right();
    input.delete_forward();
    assert_eq!(input.value(), "ac");
    assert_eq!(input.cursor(), 1);
}

// ── movement ─────────────────────────────────────────────────────

#[test]
fn move_left_at_start_is_noop() {
    let mut input = TextInput::with_value("abc");
    input.move_home();
    input.move_left();
    assert_eq!(input.cursor(), 0);
}

#[test]
fn move_right_at_end_is_noop() {
    let mut input = TextInput::with_value("abc");
    input.move_right();
    assert_eq!(input.cursor(), 3);
}

#[test]
fn move_left_decrements_cursor() {
    let mut input = TextInput::with_value("abc");
    input.move_left();
    assert_eq!(input.cursor(), 2);
}

#[test]
fn move_right_increments_cursor() {
    let mut input = TextInput::with_value("abc");
    input.move_home();
    input.move_right();
    assert_eq!(input.cursor(), 1);
}

#[test]
fn move_home_goes_to_start() {
    let mut input = TextInput::with_value("abc");
    input.move_home();
    assert_eq!(input.cursor(), 0);
}

#[test]
fn move_end_goes_to_end() {
    let mut input = TextInput::with_value("abc");
    input.move_home();
    input.move_end();
    assert_eq!(input.cursor(), 3);
}

// ── clear ────────────────────────────────────────────────────────

#[test]
fn clear_empties_content_and_resets_cursor() {
    let mut input = TextInput::with_value("abc");
    input.clear();
    assert_eq!(input.value(), "");
    assert_eq!(input.cursor(), 0);
}

// ── Unicode ──────────────────────────────────────────────────────

#[test]
fn unicode_insert_and_cursor_counts_chars() {
    let mut input = TextInput::new();
    input.insert('你');
    input.insert('好');
    assert_eq!(input.value(), "你好");
    assert_eq!(input.cursor(), 2);
}

#[test]
fn unicode_delete_back() {
    let mut input = TextInput::with_value("你好世界");
    input.delete_back();
    assert_eq!(input.value(), "你好世");
    assert_eq!(input.cursor(), 3);
}

#[test]
fn unicode_move_left_and_right() {
    let mut input = TextInput::with_value("abc你好");
    input.move_left();
    input.move_left();
    assert_eq!(input.cursor(), 3);
    input.move_right();
    assert_eq!(input.cursor(), 4);
}

#[test]
fn unicode_insert_in_middle() {
    let mut input = TextInput::with_value("你界");
    input.move_left();
    input.insert('好');
    assert_eq!(input.value(), "你好界");
}

#[test]
fn unicode_delete_forward() {
    let mut input = TextInput::with_value("你好世界");
    input.move_home();
    input.delete_forward();
    assert_eq!(input.value(), "好世界");
    assert_eq!(input.cursor(), 0);
}

// ── Empty input edge cases ───────────────────────────────────────

#[test]
fn empty_input_delete_back_is_noop() {
    let mut input = TextInput::new();
    input.delete_back();
    assert_eq!(input.value(), "");
}

#[test]
fn empty_input_delete_forward_is_noop() {
    let mut input = TextInput::new();
    input.delete_forward();
    assert_eq!(input.value(), "");
}

#[test]
fn empty_input_move_left_is_noop() {
    let mut input = TextInput::new();
    input.move_left();
    assert_eq!(input.cursor(), 0);
}

#[test]
fn empty_input_move_right_is_noop() {
    let mut input = TextInput::new();
    input.move_right();
    assert_eq!(input.cursor(), 0);
}
