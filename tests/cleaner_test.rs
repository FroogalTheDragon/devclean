//! Tests for the cleaner module: dry-run, actual deletion, error handling, multi-project cleaning.

use std::fs;
use std::path::{Path, PathBuf};

use dev_sweep::cleaner::{clean_project, clean_projects};
use dev_sweep::scanner::walk::analyze_project;
use dev_sweep::scanner::ProjectKind;

/// Helper: create a fresh temp dir for a test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dev_sweep_test_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Helper: create a Rust project with a target/ directory containing some data.
fn create_rust_project(root: &Path) {
    fs::write(root.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::create_dir_all(root.join("target/debug")).unwrap();
    fs::write(root.join("target/debug/app"), "binary_content_here").unwrap();
    fs::write(root.join("target/debug/app.d"), "deps").unwrap();
}

// ── dry_run ─────────────────────────────────────────────────────────────────

#[test]
fn dry_run_does_not_delete() {
    let dir = test_dir("clean_dryrun");
    create_rust_project(&dir);

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.total_cleanable_bytes > 0);

    let result = clean_project(&project, true).unwrap();

    // Dry run should report what would be cleaned
    assert_eq!(result.targets_cleaned, 1);
    assert!(result.bytes_freed > 0);
    assert!(result.errors.is_empty());

    // But the directory should still exist
    assert!(dir.join("target").exists());
    assert!(dir.join("target/debug/app").exists());

    fs::remove_dir_all(&dir).unwrap();
}

// ── actual deletion ─────────────────────────────────────────────────────────

#[test]
fn clean_actually_removes_target() {
    let dir = test_dir("clean_real");
    create_rust_project(&dir);

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    let result = clean_project(&project, false).unwrap();

    assert_eq!(result.targets_cleaned, 1);
    assert!(result.bytes_freed > 0);
    assert!(result.errors.is_empty());

    // target/ should be gone
    assert!(!dir.join("target").exists());

    // Source files should be untouched
    assert!(dir.join("Cargo.toml").exists());
    assert!(dir.join("src/main.rs").exists());

    fs::remove_dir_all(&dir).unwrap();
}

// ── error handling ──────────────────────────────────────────────────────────

#[test]
fn clean_nonexistent_target_reports_error() {
    let dir = test_dir("clean_noexist");
    create_rust_project(&dir);

    let mut project = analyze_project(&dir, ProjectKind::Rust).unwrap();

    // Manually break the path so deletion fails
    project.clean_targets[0].path = dir.join("target_does_not_exist");

    let result = clean_project(&project, false).unwrap();

    assert_eq!(result.targets_cleaned, 0);
    assert_eq!(result.bytes_freed, 0);
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].contains("target_does_not_exist"));

    fs::remove_dir_all(&dir).unwrap();
}

// ── multi-project cleaning ──────────────────────────────────────────────────

#[test]
fn clean_multiple_projects_dry_run() {
    let root = test_dir("clean_multi_dry");

    let proj1 = root.join("app1");
    fs::create_dir_all(&proj1).unwrap();
    create_rust_project(&proj1);

    let proj2 = root.join("app2");
    fs::create_dir_all(&proj2).unwrap();
    create_rust_project(&proj2);

    let p1 = analyze_project(&proj1, ProjectKind::Rust).unwrap();
    let p2 = analyze_project(&proj2, ProjectKind::Rust).unwrap();

    let projects: Vec<&_> = vec![&p1, &p2];
    let results = clean_projects(&projects, true);

    assert_eq!(results.len(), 2);
    let total_freed: u64 = results.iter().map(|r| r.bytes_freed).sum();
    assert!(total_freed > 0);

    // Both targets should still exist (dry run)
    assert!(proj1.join("target").exists());
    assert!(proj2.join("target").exists());

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn clean_multiple_projects_for_real() {
    let root = test_dir("clean_multi_real");

    let proj1 = root.join("app1");
    fs::create_dir_all(&proj1).unwrap();
    create_rust_project(&proj1);

    let proj2 = root.join("app2");
    fs::create_dir_all(&proj2).unwrap();
    create_rust_project(&proj2);

    let p1 = analyze_project(&proj1, ProjectKind::Rust).unwrap();
    let p2 = analyze_project(&proj2, ProjectKind::Rust).unwrap();

    let projects: Vec<&_> = vec![&p1, &p2];
    let results = clean_projects(&projects, false);

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.errors.is_empty()));

    // Both targets should be gone
    assert!(!proj1.join("target").exists());
    assert!(!proj2.join("target").exists());

    // Source files untouched
    assert!(proj1.join("Cargo.toml").exists());
    assert!(proj2.join("Cargo.toml").exists());

    fs::remove_dir_all(&root).unwrap();
}

// ── edge case: project with no clean targets ────────────────────────────────

#[test]
fn clean_project_with_no_targets() {
    let dir = test_dir("clean_notargets");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.clean_targets.is_empty());

    let result = clean_project(&project, false).unwrap();
    assert_eq!(result.targets_cleaned, 0);
    assert_eq!(result.bytes_freed, 0);
    assert!(result.errors.is_empty());

    fs::remove_dir_all(&dir).unwrap();
}
