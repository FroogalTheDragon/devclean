//! Tests for display utilities: byte formatting, ANSI handling, padding, age formatting, truncation.

use dev_sweep::tui::display::{blue, bold, cyan, dim, green, green_bold, red, yellow};
use dev_sweep::util::{
    format_age, format_bytes, pad_left, pad_right, shorten_path, truncate, visible_len,
};

// ── format_bytes ────────────────────────────────────────────────────────────

#[test]
fn format_bytes_zero() {
    assert_eq!(format_bytes(0), "0 B");
}

#[test]
fn format_bytes_small() {
    assert_eq!(format_bytes(1), "1 B");
    assert_eq!(format_bytes(512), "512 B");
    assert_eq!(format_bytes(1023), "1023 B");
}

#[test]
fn format_bytes_kilobytes() {
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(10 * 1024), "10.0 KB");
}

#[test]
fn format_bytes_megabytes() {
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(format_bytes(500 * 1024 * 1024), "500.0 MB");
}

#[test]
fn format_bytes_gigabytes() {
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(format_bytes(5 * 1024 * 1024 * 1024), "5.0 GB");
}

#[test]
fn format_bytes_terabytes() {
    assert_eq!(format_bytes(1024u64 * 1024 * 1024 * 1024), "1.0 TB");
}

#[test]
fn format_bytes_boundary() {
    // Exactly at the KB boundary
    assert_eq!(format_bytes(1024), "1.0 KB");
    // Just below KB
    assert_eq!(format_bytes(1023), "1023 B");
}

// ── visible_len ─────────────────────────────────────────────────────────────

#[test]
fn visible_len_plain_text() {
    assert_eq!(visible_len("hello"), 5);
    assert_eq!(visible_len(""), 0);
    assert_eq!(visible_len("abc def"), 7);
    assert_eq!(visible_len(" "), 1);
}

#[test]
fn visible_len_with_bold() {
    assert_eq!(visible_len(&bold("hi")), 2);
}

#[test]
fn visible_len_with_cyan() {
    assert_eq!(visible_len(&cyan("Rust")), 4);
}

#[test]
fn visible_len_with_green_bold() {
    assert_eq!(visible_len(&green_bold("✓")), 1);
}

#[test]
fn visible_len_mixed_text_and_ansi() {
    let s = format!("hello {}", cyan("world"));
    assert_eq!(visible_len(&s), 11); // "hello world"
}

#[test]
fn visible_len_multiple_ansi_sequences() {
    let s = format!("{} {} {}", red("a"), green("b"), blue("c"));
    assert_eq!(visible_len(&s), 5); // "a b c"
}

#[test]
fn visible_len_dim() {
    assert_eq!(visible_len(&dim("faded")), 5);
}

#[test]
fn visible_len_yellow() {
    assert_eq!(visible_len(&yellow("warning")), 7);
}

// ── pad_right ───────────────────────────────────────────────────────────────

#[test]
fn pad_right_adds_spaces() {
    assert_eq!(pad_right("hi", 5), "hi   ");
}

#[test]
fn pad_right_exact_width() {
    assert_eq!(pad_right("hello", 5), "hello");
}

#[test]
fn pad_right_no_truncation_on_overflow() {
    assert_eq!(pad_right("toolong", 3), "toolong");
}

#[test]
fn pad_right_empty_string() {
    assert_eq!(pad_right("", 3), "   ");
}

#[test]
fn pad_right_with_ansi() {
    let colored = cyan("Rust"); // visible len 4
    let padded = pad_right(&colored, 8);
    assert_eq!(visible_len(&padded), 8);
    // Should end with 4 spaces
    assert!(padded.ends_with("    "));
}

// ── pad_left ────────────────────────────────────────────────────────────────

#[test]
fn pad_left_adds_spaces() {
    assert_eq!(pad_left("42", 5), "   42");
}

#[test]
fn pad_left_exact_width() {
    assert_eq!(pad_left("hello", 5), "hello");
}

#[test]
fn pad_left_no_truncation_on_overflow() {
    assert_eq!(pad_left("toolong", 3), "toolong");
}

#[test]
fn pad_left_empty_string() {
    assert_eq!(pad_left("", 3), "   ");
}

#[test]
fn pad_left_with_ansi() {
    let colored = yellow("9.1 GB"); // visible len 6
    let padded = pad_left(&colored, 10);
    assert_eq!(visible_len(&padded), 10);
    // Should start with 4 spaces
    assert!(padded.starts_with("    "));
}

// ── format_age ──────────────────────────────────────────────────────────────

#[test]
fn format_age_just_now() {
    let d = chrono::TimeDelta::try_minutes(5).unwrap();
    assert_eq!(format_age(d), "just now");
}

#[test]
fn format_age_zero() {
    let d = chrono::TimeDelta::try_seconds(0).unwrap();
    assert_eq!(format_age(d), "just now");
}

#[test]
fn format_age_hours() {
    let d = chrono::TimeDelta::try_hours(3).unwrap();
    assert_eq!(format_age(d), "3h ago");
}

#[test]
fn format_age_one_hour() {
    let d = chrono::TimeDelta::try_hours(1).unwrap();
    assert_eq!(format_age(d), "1h ago");
}

#[test]
fn format_age_days() {
    let d = chrono::TimeDelta::try_days(15).unwrap();
    assert_eq!(format_age(d), "15d ago");
}

#[test]
fn format_age_one_day() {
    let d = chrono::TimeDelta::try_days(1).unwrap();
    assert_eq!(format_age(d), "1d ago");
}

#[test]
fn format_age_months() {
    let d = chrono::TimeDelta::try_days(90).unwrap();
    assert_eq!(format_age(d), "3mo ago");
}

#[test]
fn format_age_years() {
    let d = chrono::TimeDelta::try_days(400).unwrap();
    assert_eq!(format_age(d), "1.1y ago");
}

#[test]
fn format_age_exactly_one_year() {
    let d = chrono::TimeDelta::try_days(366).unwrap();
    assert_eq!(format_age(d), "1.0y ago");
}

// ── truncate ────────────────────────────────────────────────────────────────

#[test]
fn truncate_short_string() {
    assert_eq!(truncate("short", 10), "short");
}

#[test]
fn truncate_exact_width() {
    assert_eq!(truncate("exact", 5), "exact");
}

#[test]
fn truncate_long_string() {
    assert_eq!(truncate("hello world", 8), "hello w…");
}

#[test]
fn truncate_width_one() {
    assert_eq!(truncate("hello", 1), "…");
}

#[test]
fn truncate_empty_string() {
    assert_eq!(truncate("", 5), "");
}

#[test]
fn truncate_width_two() {
    assert_eq!(truncate("hello", 2), "h…");
}

// ── shorten_path ────────────────────────────────────────────────────────────

#[test]
fn shorten_path_with_home() {
    if let Some(home) = dirs::home_dir() {
        let path = format!("{}/projects/dev-sweep", home.display());
        let shortened = shorten_path(&path);
        assert!(shortened.starts_with("~/"));
        assert!(shortened.ends_with("projects/dev-sweep"));
        assert!(!shortened.contains(&home.display().to_string()));
    }
}

#[test]
fn shorten_path_outside_home() {
    let path = "/tmp/some/random/path";
    assert_eq!(shorten_path(path), path);
}

#[test]
fn shorten_path_root() {
    assert_eq!(shorten_path("/"), "/");
}

// ── ANSI helper functions produce correct sequences ─────────────────────────

#[test]
fn ansi_helpers_wrap_correctly() {
    assert!(bold("x").starts_with("\x1b["));
    assert!(bold("x").ends_with("\x1b[0m"));
    assert!(cyan("x").contains("x"));
    assert!(green("x").contains("x"));
    assert!(red("x").contains("x"));
    assert!(dim("x").contains("x"));
    assert!(blue("x").contains("x"));
}
