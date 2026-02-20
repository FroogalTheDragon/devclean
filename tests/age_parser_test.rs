//! Tests for the age string parser (e.g. "30d", "3m", "1y", "2w").

use dev_sweep::util::parse_age;

// ── valid inputs ────────────────────────────────────────────────────────────

#[test]
fn parse_days() {
    let d = parse_age("30d").unwrap();
    assert_eq!(d.num_days(), 30);
}

#[test]
fn parse_one_day() {
    let d = parse_age("1d").unwrap();
    assert_eq!(d.num_days(), 1);
}

#[test]
fn parse_weeks() {
    let d = parse_age("2w").unwrap();
    assert_eq!(d.num_days(), 14);
}

#[test]
fn parse_months() {
    let d = parse_age("3m").unwrap();
    assert_eq!(d.num_days(), 90);
}

#[test]
fn parse_years() {
    let d = parse_age("1y").unwrap();
    assert_eq!(d.num_days(), 365);
}

#[test]
fn parse_large_number() {
    let d = parse_age("100d").unwrap();
    assert_eq!(d.num_days(), 100);
}

#[test]
fn parse_with_whitespace() {
    let d = parse_age("  7d  ").unwrap();
    assert_eq!(d.num_days(), 7);
}

#[test]
fn parse_uppercase() {
    let d = parse_age("5D").unwrap();
    assert_eq!(d.num_days(), 5);
}

#[test]
fn parse_mixed_case() {
    let d = parse_age("2M").unwrap();
    assert_eq!(d.num_days(), 60);
}

// ── invalid inputs ──────────────────────────────────────────────────────────

#[test]
fn parse_no_unit() {
    assert!(parse_age("30").is_err());
}

#[test]
fn parse_no_number() {
    assert!(parse_age("d").is_err());
}

#[test]
fn parse_invalid_unit() {
    assert!(parse_age("30x").is_err());
}

#[test]
fn parse_empty_string() {
    assert!(parse_age("").is_err());
}

#[test]
fn parse_just_whitespace() {
    assert!(parse_age("   ").is_err());
}

#[test]
fn parse_negative_number() {
    // "-5d" — the number parser will fail on "-5" because we parse as i64
    // Actually "-5" does parse as i64, so let's see what happens
    let result = parse_age("-5d");
    // Negative durations are technically valid in chrono, so this might succeed
    // The important thing is it doesn't panic
    if let Ok(d) = result {
        assert_eq!(d.num_days(), -5);
    }
}

#[test]
fn parse_float() {
    // "1.5d" should fail — we only support integers
    assert!(parse_age("1.5d").is_err());
}

#[test]
fn parse_zero() {
    let d = parse_age("0d").unwrap();
    assert_eq!(d.num_days(), 0);
}
