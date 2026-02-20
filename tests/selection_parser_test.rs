//! Tests for the multi-select input parser (e.g. "1,3,5-8").

use dev_sweep::tui::display::parse_selection;

// ── valid inputs ────────────────────────────────────────────────────────────

#[test]
fn parse_single_number() {
    let result = parse_selection("3", 10).unwrap();
    assert_eq!(result, vec![2]); // 1-indexed → 0-indexed
}

#[test]
fn parse_comma_separated() {
    let result = parse_selection("1,3,5", 10).unwrap();
    assert_eq!(result, vec![0, 2, 4]);
}

#[test]
fn parse_space_separated() {
    let result = parse_selection("1 3 5", 10).unwrap();
    assert_eq!(result, vec![0, 2, 4]);
}

#[test]
fn parse_range() {
    let result = parse_selection("2-5", 10).unwrap();
    assert_eq!(result, vec![1, 2, 3, 4]);
}

#[test]
fn parse_mixed() {
    let result = parse_selection("1,3-5,8", 10).unwrap();
    assert_eq!(result, vec![0, 2, 3, 4, 7]);
}

#[test]
fn parse_deduplicates() {
    let result = parse_selection("1,1,2,2", 10).unwrap();
    assert_eq!(result, vec![0, 1]);
}

#[test]
fn parse_sorts_output() {
    let result = parse_selection("5,1,3", 10).unwrap();
    assert_eq!(result, vec![0, 2, 4]);
}

#[test]
fn parse_single_item_range() {
    let result = parse_selection("3-3", 10).unwrap();
    assert_eq!(result, vec![2]);
}

#[test]
fn parse_empty_returns_empty() {
    let result = parse_selection("", 10).unwrap();
    assert!(result.is_empty());
}

#[test]
fn parse_whitespace_around_numbers() {
    let result = parse_selection(" 1 , 3 ", 10).unwrap();
    assert_eq!(result, vec![0, 2]);
}

// ── invalid inputs ──────────────────────────────────────────────────────────

#[test]
fn parse_zero_is_error() {
    assert!(parse_selection("0", 10).is_err());
}

#[test]
fn parse_exceeds_max_is_error() {
    assert!(parse_selection("11", 10).is_err());
}

#[test]
fn parse_reversed_range_is_error() {
    assert!(parse_selection("5-2", 10).is_err());
}

#[test]
fn parse_range_exceeds_max_is_error() {
    assert!(parse_selection("8-12", 10).is_err());
}

#[test]
fn parse_non_number_is_error() {
    assert!(parse_selection("abc", 10).is_err());
}

#[test]
fn parse_range_with_non_number_is_error() {
    assert!(parse_selection("a-5", 10).is_err());
}
