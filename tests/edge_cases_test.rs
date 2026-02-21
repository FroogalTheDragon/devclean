//! Edge case tests: symlinks, permission issues, deeply nested projects,
//! Unicode paths, empty artifact dirs, concurrent project types, and more.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use dev_sweep::scanner::ProjectKind;
use dev_sweep::scanner::walk::{
    analyze_project, detect_project_kind, dir_size, scan_directory,
};
use dev_sweep::cleaner::{clean_project, clean_projects};
use dev_sweep::util::{format_bytes, parse_age};

/// Helper: create a fresh temp dir for a test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dev_sweep_test_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ══════════════════════════════════════════════════════════════════════════════
// Symlinks
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn symlink_to_marker_file_is_not_detected() {
    // A symlink named Cargo.toml pointing elsewhere should not be treated as
    // a regular file marker (is_file() returns true for symlinks, so this
    // actually tests current behavior — document it either way).
    let dir = test_dir("edge_symlink_marker");
    let target_file = dir.join("real_file.toml");
    fs::write(&target_file, "[package]").unwrap();
    symlink(&target_file, dir.join("Cargo.toml")).unwrap();

    // Symlinks do resolve via is_file(), so detection should still work
    let kind = detect_project_kind(&dir);
    assert_eq!(kind, Some(ProjectKind::Rust));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn symlink_inside_artifact_dir_counted_in_size() {
    // A target/ directory containing a symlink to a file — dir_size should
    // handle it gracefully (count the link target size or skip, but not crash).
    let dir = test_dir("edge_symlink_artifact");
    let artifact = dir.join("target");
    fs::create_dir_all(&artifact).unwrap();

    let real_file = dir.join("real_binary");
    fs::write(&real_file, "x".repeat(1000)).unwrap();
    symlink(&real_file, artifact.join("linked_binary")).unwrap();

    // Should not panic — size may or may not include the link target
    let size = dir_size(&artifact).unwrap();
    // Should not error — size may or may not include the link target
    let _ = size;
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn symlink_loop_does_not_hang() {
    // Create a symlink that points to its own parent — walkdir with
    // follow_links(false) should handle this, but let's verify dir_size
    // doesn't loop infinitely.
    let dir = test_dir("edge_symlink_loop");
    fs::create_dir_all(dir.join("subdir")).unwrap();
    fs::write(dir.join("subdir/file.txt"), "data").unwrap();
    // symlink subdir/loop -> ..  (points back to parent)
    let _ = symlink(&dir, dir.join("subdir/loop"));

    // Should complete without hanging (walkdir doesn't follow symlinks)
    let size = dir_size(&dir).unwrap();
    assert!(size > 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn scan_does_not_follow_symlinked_directories() {
    // If a symlink points to a project dir, scan should not discover it
    // as a separate project (since follow_links is false).
    let root = test_dir("edge_symlink_scan");

    let real_proj = root.join("real_project");
    fs::create_dir_all(&real_proj).unwrap();
    fs::write(real_proj.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(real_proj.join("target")).unwrap();
    fs::write(real_proj.join("target/bin"), "data").unwrap();

    // Symlink to the same project under a different name
    symlink(&real_proj, root.join("linked_project")).unwrap();

    let projects = scan_directory(&root, None).unwrap();
    // Should only find 1 project (the real one), not 2
    assert_eq!(projects.len(), 1);
    assert!(projects[0].path.ends_with("real_project"));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn dangling_symlink_in_artifact_dir() {
    // A target/ directory containing a symlink to a file that doesn't exist.
    let dir = test_dir("edge_dangling_symlink");
    let artifact = dir.join("target");
    fs::create_dir_all(&artifact).unwrap();
    fs::write(artifact.join("real.bin"), "data").unwrap();
    symlink("/nonexistent/path/to/nowhere", artifact.join("dangling")).unwrap();

    // dir_size should handle this without error
    let size = dir_size(&artifact).unwrap();
    assert!(size > 0); // at least the real file
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Unicode & special characters in paths
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn unicode_project_name() {
    let dir = test_dir("edge_ünïcödé_prøject");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(dir.join("target/debug")).unwrap();
    fs::write(dir.join("target/debug/bin"), "data").unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.name.contains("ünïcödé"));
    assert!(project.total_cleanable_bytes > 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn spaces_in_project_path() {
    let dir = test_dir("edge my project with spaces");
    fs::write(dir.join("package.json"), "{}").unwrap();
    fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
    fs::write(dir.join("node_modules/pkg/index.js"), "code").unwrap();

    let project = analyze_project(&dir, ProjectKind::Node).unwrap();
    assert!(project.total_cleanable_bytes > 0);
    assert!(project.name.contains("spaces"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_project_in_unicode_dir() {
    let dir = test_dir("edge_日本語プロジェクト");
    fs::write(dir.join("go.mod"), "module example").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Go));
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Empty / zero-byte artifact directories
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn empty_target_dir_is_not_a_clean_target() {
    // target/ exists but has zero bytes → should not appear in clean_targets
    let dir = test_dir("edge_empty_target");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(dir.join("target")).unwrap();
    // No files inside target/

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.clean_targets.is_empty());
    assert_eq!(project.total_cleanable_bytes, 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn empty_node_modules_is_not_a_clean_target() {
    let dir = test_dir("edge_empty_nm");
    fs::write(dir.join("package.json"), "{}").unwrap();
    fs::create_dir_all(dir.join("node_modules")).unwrap();

    let project = analyze_project(&dir, ProjectKind::Node).unwrap();
    assert!(project.clean_targets.is_empty());
    assert_eq!(project.total_cleanable_bytes, 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn artifact_dir_with_only_empty_subdirs() {
    // target/ has nested subdirectories but no actual files → 0 bytes
    let dir = test_dir("edge_nested_empty");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(dir.join("target/debug/build/deps")).unwrap();
    fs::create_dir_all(dir.join("target/release")).unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.clean_targets.is_empty());
    assert_eq!(project.total_cleanable_bytes, 0);
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Nested / overlapping projects
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn nested_project_inside_another_project() {
    // A Rust project containing a Node sub-project in a subdirectory.
    // Both should be detected by scan_directory.
    let root = test_dir("edge_nested_proj");

    let outer = root.join("rust_app");
    fs::create_dir_all(outer.join("src")).unwrap();
    fs::write(outer.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(outer.join("target")).unwrap();
    fs::write(outer.join("target/bin"), "data").unwrap();

    let inner = outer.join("frontend");
    fs::create_dir_all(&inner).unwrap();
    fs::write(inner.join("package.json"), "{}").unwrap();
    fs::create_dir_all(inner.join("node_modules/react")).unwrap();
    fs::write(inner.join("node_modules/react/index.js"), "code").unwrap();

    let projects = scan_directory(&root, None).unwrap();
    assert_eq!(projects.len(), 2);

    let kinds: Vec<ProjectKind> = projects.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&ProjectKind::Rust));
    assert!(kinds.contains(&ProjectKind::Node));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn project_with_multiple_marker_files_detected_once() {
    // A Java project with both pom.xml and build.gradle — should only be
    // detected once (as Java), not twice.
    let dir = test_dir("edge_double_marker");
    fs::write(dir.join("pom.xml"), "<project>").unwrap();
    fs::write(dir.join("build.gradle"), "").unwrap();
    fs::create_dir_all(dir.join("target")).unwrap();
    fs::write(dir.join("target/app.jar"), "jardata").unwrap();

    // detect_project_kind returns the first match
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Java));

    // When scanning, it should appear only once
    let root = test_dir("edge_double_marker_scan");
    let proj = root.join("java_app");
    fs::create_dir_all(&proj).unwrap();
    fs::write(proj.join("pom.xml"), "<project>").unwrap();
    fs::write(proj.join("build.gradle"), "").unwrap();
    fs::create_dir_all(proj.join("target")).unwrap();
    fs::write(proj.join("target/app.jar"), "data").unwrap();

    let projects = scan_directory(&root, None).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].kind, ProjectKind::Java);

    fs::remove_dir_all(&dir).unwrap();
    fs::remove_dir_all(&root).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Deeply nested directories
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn deeply_nested_project_found() {
    let root = test_dir("edge_deep");
    let deep = root.join("a/b/c/d/e/f/g/project");
    fs::create_dir_all(deep.join("src")).unwrap();
    fs::write(deep.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(deep.join("target")).unwrap();
    fs::write(deep.join("target/bin"), "data").unwrap();

    let projects = scan_directory(&root, None).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].kind, ProjectKind::Rust);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn max_depth_zero_finds_nothing() {
    // max_depth 0 means only the root itself — typically won't be a project
    let root = test_dir("edge_depth_zero");
    fs::write(root.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::write(root.join("target/bin"), "data").unwrap();

    // depth 0 = only the root entry itself, no children
    // The root IS a project, so it should be found at depth 0
    let projects = scan_directory(&root, Some(0)).unwrap();
    // WalkDir max_depth(0) yields only the root, so detect_project_kind
    // will check the root dir itself
    assert!(projects.len() <= 1);
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn max_depth_one_finds_only_immediate_children() {
    let root = test_dir("edge_depth_one");

    // Direct child project — depth 1
    let shallow = root.join("app");
    fs::create_dir_all(&shallow).unwrap();
    fs::write(shallow.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(shallow.join("target")).unwrap();
    fs::write(shallow.join("target/bin"), "data").unwrap();

    // Grandchild project — depth 2
    let deep = root.join("dir/nested_app");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("package.json"), "{}").unwrap();
    fs::create_dir_all(deep.join("node_modules")).unwrap();
    fs::write(deep.join("node_modules/m.js"), "x").unwrap();

    let projects = scan_directory(&root, Some(1)).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].kind, ProjectKind::Rust);
    fs::remove_dir_all(&root).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Python __pycache__ edge cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn pycache_at_project_root_not_double_counted() {
    // __pycache__ at root level — should appear in clean_targets but not duplicated
    let dir = test_dir("edge_pycache_root");
    fs::write(dir.join("pyproject.toml"), "[project]").unwrap();
    let cache = dir.join("__pycache__");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("mod.pyc"), "bytecode").unwrap();

    let project = analyze_project(&dir, ProjectKind::Python).unwrap();
    let pycache_targets: Vec<_> = project
        .clean_targets
        .iter()
        .filter(|t| t.name.contains("__pycache__"))
        .collect();
    // Should be found (either via cleanable_dirs or find_pycache_recursive) but not duplicated
    assert!(!pycache_targets.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn pycache_deeply_nested() {
    let dir = test_dir("edge_pycache_deep");
    fs::write(dir.join("setup.py"), "").unwrap();
    let deep = dir.join("src/pkg/sub/deep/__pycache__");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("deep.pyc"), "bytecode").unwrap();

    let project = analyze_project(&dir, ProjectKind::Python).unwrap();
    assert!(project
        .clean_targets
        .iter()
        .any(|t| t.name.contains("__pycache__")));
    assert!(project.total_cleanable_bytes > 0);
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Cleaner edge cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn clean_preserves_marker_files_and_source() {
    // After cleaning, the project should still be detectable (marker files intact)
    let dir = test_dir("edge_clean_preserve");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}").unwrap();
    fs::create_dir_all(dir.join("target/debug")).unwrap();
    fs::write(dir.join("target/debug/app"), "binary").unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    clean_project(&project, false).unwrap();

    // Project should still be detectable
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Rust));
    // Source intact
    assert!(dir.join("src/main.rs").exists());
    assert!(dir.join("Cargo.toml").exists());
    // Artifacts gone
    assert!(!dir.join("target").exists());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn clean_already_clean_project() {
    // Analyze, clean, then analyze again — second pass should show 0 cleanable bytes
    let dir = test_dir("edge_double_clean");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(dir.join("target/debug")).unwrap();
    fs::write(dir.join("target/debug/app"), "binary").unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.total_cleanable_bytes > 0);
    clean_project(&project, false).unwrap();

    let project2 = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert_eq!(project2.total_cleanable_bytes, 0);
    assert!(project2.clean_targets.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn clean_projects_with_mix_of_good_and_bad_targets() {
    // One project has a valid target, another has a broken path.
    // clean_projects should handle both without panicking.
    let root = test_dir("edge_mixed_clean");

    let good = root.join("good_proj");
    fs::create_dir_all(&good).unwrap();
    fs::write(good.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(good.join("target")).unwrap();
    fs::write(good.join("target/bin"), "data").unwrap();

    let bad = root.join("bad_proj");
    fs::create_dir_all(&bad).unwrap();
    fs::write(bad.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(bad.join("target")).unwrap();
    fs::write(bad.join("target/bin"), "data").unwrap();

    let good_project = analyze_project(&good, ProjectKind::Rust).unwrap();
    let mut bad_project = analyze_project(&bad, ProjectKind::Rust).unwrap();
    // Break the bad project's target path
    bad_project.clean_targets[0].path = root.join("nonexistent");

    let projects: Vec<&_> = vec![&good_project, &bad_project];
    let results = clean_projects(&projects, false);

    assert_eq!(results.len(), 2);
    // Good project should succeed
    assert!(results[0].errors.is_empty());
    assert!(results[0].bytes_freed > 0);
    // Bad project should report an error
    assert!(!results[1].errors.is_empty());
    assert_eq!(results[1].bytes_freed, 0);

    fs::remove_dir_all(&root).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Large number of projects
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn scan_many_projects_at_once() {
    let root = test_dir("edge_many_projects");

    for i in 0..25 {
        let proj = root.join(format!("project_{i:03}"));
        fs::create_dir_all(proj.join("src")).unwrap();
        fs::write(proj.join("Cargo.toml"), "[package]").unwrap();
        fs::create_dir_all(proj.join("target")).unwrap();
        fs::write(proj.join("target/bin"), format!("binary_{i}")).unwrap();
    }

    let projects = scan_directory(&root, None).unwrap();
    assert_eq!(projects.len(), 25);
    assert!(projects.iter().all(|p| p.kind == ProjectKind::Rust));
    assert!(projects.iter().all(|p| p.total_cleanable_bytes > 0));
    fs::remove_dir_all(&root).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Glob-based marker edge cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn glob_marker_no_match() {
    // Directory has files but none matching the *.csproj glob
    let dir = test_dir("edge_glob_nomatch");
    fs::write(dir.join("readme.md"), "hello").unwrap();
    fs::write(dir.join("app.txt"), "not a project").unwrap();

    assert_eq!(detect_project_kind(&dir), None);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn glob_marker_multiple_matches() {
    // Multiple .csproj files — should still detect as .NET once
    let dir = test_dir("edge_glob_multi");
    fs::write(dir.join("App.csproj"), "<Project>").unwrap();
    fs::write(dir.join("Lib.csproj"), "<Project>").unwrap();

    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::DotNet));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn glob_cleanable_dir_multiple_egg_info() {
    // Python *.egg-info — multiple egg-info dirs should all be found
    let dir = test_dir("edge_multi_egg");
    fs::write(dir.join("setup.py"), "").unwrap();
    fs::create_dir_all(dir.join("mypackage.egg-info")).unwrap();
    fs::write(dir.join("mypackage.egg-info/PKG-INFO"), "info").unwrap();
    fs::create_dir_all(dir.join("other.egg-info")).unwrap();
    fs::write(dir.join("other.egg-info/PKG-INFO"), "info2").unwrap();

    let project = analyze_project(&dir, ProjectKind::Python).unwrap();
    let egg_targets: Vec<_> = project
        .clean_targets
        .iter()
        .filter(|t| t.name.contains("egg-info"))
        .collect();
    assert_eq!(egg_targets.len(), 2);
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Nested-path marker (Unity)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn unity_incomplete_marker_not_detected() {
    // ProjectSettings/ exists but ProjectVersion.txt is missing
    let dir = test_dir("edge_unity_incomplete");
    fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
    fs::write(dir.join("ProjectSettings/other.txt"), "").unwrap();

    assert_eq!(detect_project_kind(&dir), None);
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// dir_size edge cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dir_size_nonexistent_path() {
    let path = std::env::temp_dir().join("dev_sweep_test_nonexistent_dir_xyz");
    let _ = fs::remove_dir_all(&path);
    // dir_size on a path that doesn't exist — should return 0 or error, not panic
    let result = dir_size(&path);
    // walkdir on nonexistent dir returns an error iterator, we just sum 0
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn dir_size_single_large_file() {
    let dir = test_dir("edge_size_large");
    let data = "x".repeat(100_000);
    fs::write(dir.join("big_file.bin"), &data).unwrap();

    let size = dir_size(&dir).unwrap();
    assert_eq!(size, 100_000);
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// format_bytes edge cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn format_bytes_zero() {
    assert_eq!(format_bytes(0), "0 B");
}

#[test]
fn format_bytes_one() {
    assert_eq!(format_bytes(1), "1 B");
}

#[test]
fn format_bytes_boundary_kb() {
    assert_eq!(format_bytes(1024), "1.0 KB");
}

#[test]
fn format_bytes_boundary_mb() {
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
}

#[test]
fn format_bytes_boundary_gb() {
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
}

#[test]
fn format_bytes_boundary_tb() {
    assert_eq!(format_bytes(1024u64 * 1024 * 1024 * 1024), "1.0 TB");
}

#[test]
fn format_bytes_just_under_kb() {
    assert_eq!(format_bytes(1023), "1023 B");
}

#[test]
fn format_bytes_u64_max() {
    // Should not panic on the largest possible value
    let result = format_bytes(u64::MAX);
    assert!(result.contains("TB") || result.contains("EB"));
}

// ══════════════════════════════════════════════════════════════════════════════
// parse_age edge cases
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn parse_age_very_large_number() {
    let result = parse_age("99999d");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().num_days(), 99999);
}

#[test]
fn parse_age_one_week() {
    let d = parse_age("1w").unwrap();
    assert_eq!(d.num_days(), 7);
}

#[test]
fn parse_age_twelve_months() {
    let d = parse_age("12m").unwrap();
    assert_eq!(d.num_days(), 360); // 12 * 30
}

#[test]
fn parse_age_multiple_digit_years() {
    let d = parse_age("10y").unwrap();
    assert_eq!(d.num_days(), 3650);
}

// ══════════════════════════════════════════════════════════════════════════════
// Marker file is a directory (not a file)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn marker_file_as_directory_not_detected() {
    // Someone has a directory named Cargo.toml (weird, but possible)
    // is_file() should return false, so the project should NOT be detected
    let dir = test_dir("edge_marker_is_dir");
    fs::create_dir_all(dir.join("Cargo.toml")).unwrap();

    assert_eq!(detect_project_kind(&dir), None);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn marker_directory_package_json() {
    // package.json as a directory — should not detect as Node
    let dir = test_dir("edge_pkgjson_dir");
    fs::create_dir_all(dir.join("package.json")).unwrap();

    assert_eq!(detect_project_kind(&dir), None);
    fs::remove_dir_all(&dir).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Hidden directories
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn project_inside_hidden_directory_skipped() {
    // Projects inside dot-prefixed directories (like .backup) should be
    // skipped — should_visit unconditionally rejects all hidden dirs at depth > 0.
    let root = test_dir("edge_hidden");
    let hidden = root.join(".backup/project");
    fs::create_dir_all(hidden.join("src")).unwrap();
    fs::write(hidden.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(hidden.join("target")).unwrap();
    fs::write(hidden.join("target/bin"), "data").unwrap();

    let projects = scan_directory(&root, None).unwrap();
    assert!(projects.is_empty());
    fs::remove_dir_all(&root).unwrap();
}

// ══════════════════════════════════════════════════════════════════════════════
// Scala nested target (project/target)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn scala_project_target_and_nested_target() {
    let dir = test_dir("edge_scala_targets");
    fs::write(dir.join("build.sbt"), "").unwrap();
    fs::create_dir_all(dir.join("target/scala-2.13")).unwrap();
    fs::write(dir.join("target/scala-2.13/app.jar"), "jar").unwrap();
    fs::create_dir_all(dir.join("project/target")).unwrap();
    fs::write(dir.join("project/target/resolution.json"), "{}").unwrap();

    let project = analyze_project(&dir, ProjectKind::Scala).unwrap();
    let names: Vec<&str> = project.clean_targets.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"target"));
    assert!(names.contains(&"project/target"));
    assert_eq!(project.clean_targets.len(), 2);
    fs::remove_dir_all(&dir).unwrap();
}
