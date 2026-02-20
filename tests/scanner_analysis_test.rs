//! Tests for project analysis, directory sizing, pycache discovery, and full scanning.

use std::fs;
use std::path::PathBuf;

use dev_sweep::scanner::ProjectKind;
use dev_sweep::scanner::walk::{
    analyze_project, dir_size, find_pycache_recursive, scan_directory, should_visit,
};

/// Helper: create a fresh temp dir for a test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dev_sweep_test_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ── dir_size ────────────────────────────────────────────────────────────────

#[test]
fn dir_size_empty() {
    let dir = test_dir("size_empty");
    assert_eq!(dir_size(&dir).unwrap(), 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn dir_size_flat_files() {
    let dir = test_dir("size_flat");
    fs::write(dir.join("a.txt"), "hello").unwrap(); // 5
    fs::write(dir.join("b.txt"), "world!").unwrap(); // 6
    assert_eq!(dir_size(&dir).unwrap(), 11);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn dir_size_nested() {
    let dir = test_dir("size_nested");
    let sub = dir.join("sub/deep");
    fs::create_dir_all(&sub).unwrap();
    fs::write(dir.join("top.txt"), "abc").unwrap(); // 3
    fs::write(sub.join("deep.txt"), "de").unwrap(); // 2
    assert_eq!(dir_size(&dir).unwrap(), 5);
    fs::remove_dir_all(&dir).unwrap();
}

// ── should_visit ────────────────────────────────────────────────────────────

#[test]
fn should_visit_skips_node_modules() {
    let dir = test_dir("visit_nm");
    fs::create_dir_all(dir.join("node_modules")).unwrap();

    for entry in walkdir::WalkDir::new(&dir).max_depth(1) {
        let entry = entry.unwrap();
        if entry.file_name() == "node_modules" {
            assert!(!should_visit(&entry));
        }
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn should_visit_skips_target() {
    let dir = test_dir("visit_target");
    fs::create_dir_all(dir.join("target")).unwrap();

    for entry in walkdir::WalkDir::new(&dir).max_depth(1) {
        let entry = entry.unwrap();
        if entry.file_name() == "target" {
            assert!(!should_visit(&entry));
        }
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn should_visit_skips_git() {
    let dir = test_dir("visit_git");
    fs::create_dir_all(dir.join(".git")).unwrap();

    for entry in walkdir::WalkDir::new(&dir).max_depth(1) {
        let entry = entry.unwrap();
        if entry.file_name() == ".git" {
            assert!(!should_visit(&entry));
        }
    }
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn should_visit_allows_src() {
    let dir = test_dir("visit_src");
    fs::create_dir_all(dir.join("src")).unwrap();

    for entry in walkdir::WalkDir::new(&dir).max_depth(1) {
        let entry = entry.unwrap();
        if entry.file_name() == "src" {
            assert!(should_visit(&entry));
        }
    }
    fs::remove_dir_all(&dir).unwrap();
}

// ── analyze_project ─────────────────────────────────────────────────────────

#[test]
fn analyze_rust_with_target() {
    let dir = test_dir("analyze_rust");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(dir.join("target/debug")).unwrap();
    fs::write(dir.join("target/debug/main"), "binary_data_here!").unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert_eq!(project.kind, ProjectKind::Rust);
    assert_eq!(project.clean_targets.len(), 1);
    assert_eq!(project.clean_targets[0].name, "target");
    assert!(project.total_cleanable_bytes > 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn analyze_node_with_node_modules() {
    let dir = test_dir("analyze_node");
    fs::write(dir.join("package.json"), "{}").unwrap();
    fs::create_dir_all(dir.join("node_modules/express")).unwrap();
    fs::write(dir.join("node_modules/express/index.js"), "module.exports = {}").unwrap();

    let project = analyze_project(&dir, ProjectKind::Node).unwrap();
    assert_eq!(project.kind, ProjectKind::Node);
    assert!(project.clean_targets.iter().any(|t| t.name == "node_modules"));
    assert!(project.total_cleanable_bytes > 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn analyze_node_with_multiple_targets() {
    let dir = test_dir("analyze_node_multi");
    fs::write(dir.join("package.json"), "{}").unwrap();
    fs::create_dir_all(dir.join("node_modules")).unwrap();
    fs::write(dir.join("node_modules/mod.js"), "x").unwrap();
    fs::create_dir_all(dir.join("dist")).unwrap();
    fs::write(dir.join("dist/bundle.js"), "code").unwrap();
    fs::create_dir_all(dir.join(".next")).unwrap();
    fs::write(dir.join(".next/cache.json"), "{}").unwrap();

    let project = analyze_project(&dir, ProjectKind::Node).unwrap();
    assert!(project.clean_targets.len() >= 3);

    let names: Vec<&str> = project.clean_targets.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"node_modules"));
    assert!(names.contains(&"dist"));
    assert!(names.contains(&".next"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn analyze_project_no_artifacts() {
    let dir = test_dir("analyze_clean");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.clean_targets.is_empty());
    assert_eq!(project.total_cleanable_bytes, 0);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn analyze_project_name_from_dirname() {
    let dir = test_dir("my_cool_project");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();

    let project = analyze_project(&dir, ProjectKind::Rust).unwrap();
    assert!(project.name.contains("my_cool_project"));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn analyze_dotnet_bin_obj() {
    let dir = test_dir("analyze_dotnet");
    fs::write(dir.join("App.csproj"), "<Project>").unwrap();
    fs::create_dir_all(dir.join("bin/Debug")).unwrap();
    fs::write(dir.join("bin/Debug/App.dll"), "dll").unwrap();
    fs::create_dir_all(dir.join("obj")).unwrap();
    fs::write(dir.join("obj/project.assets.json"), "{}").unwrap();

    let project = analyze_project(&dir, ProjectKind::DotNet).unwrap();
    let names: Vec<&str> = project.clean_targets.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"bin"));
    assert!(names.contains(&"obj"));
    fs::remove_dir_all(&dir).unwrap();
}

// ── find_pycache_recursive ──────────────────────────────────────────────────

#[test]
fn pycache_finds_nested() {
    let dir = test_dir("pycache_nested");
    let cache1 = dir.join("src/__pycache__");
    let cache2 = dir.join("src/utils/__pycache__");
    fs::create_dir_all(&cache1).unwrap();
    fs::create_dir_all(&cache2).unwrap();
    fs::write(cache1.join("mod.pyc"), "bytecode").unwrap();
    fs::write(cache2.join("helpers.pyc"), "bytecode").unwrap();

    let mut targets = Vec::new();
    find_pycache_recursive(&dir, &mut targets);

    assert_eq!(targets.len(), 2);
    let names: Vec<&str> = targets.iter().map(|t| t.name.as_str()).collect();
    assert!(names.iter().any(|n| n.contains("src/__pycache__")));
    assert!(names.iter().any(|n| n.contains("src/utils/__pycache__")));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn pycache_empty_dir() {
    let dir = test_dir("pycache_empty");
    let mut targets = Vec::new();
    find_pycache_recursive(&dir, &mut targets);
    assert!(targets.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn pycache_skips_empty_cache_dirs() {
    let dir = test_dir("pycache_no_files");
    // __pycache__ exists but has no files inside → size 0 → should be skipped
    fs::create_dir_all(dir.join("pkg/__pycache__")).unwrap();

    let mut targets = Vec::new();
    find_pycache_recursive(&dir, &mut targets);
    assert!(targets.is_empty());
    fs::remove_dir_all(&dir).unwrap();
}

// ── scan_directory (integration) ────────────────────────────────────────────

#[test]
fn scan_finds_multiple_project_types() {
    let root = test_dir("scan_multi");

    // Rust project with artifacts
    let rust_proj = root.join("my_rust_app");
    fs::create_dir_all(rust_proj.join("src")).unwrap();
    fs::write(rust_proj.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(rust_proj.join("target/debug")).unwrap();
    fs::write(rust_proj.join("target/debug/app"), "binary").unwrap();

    // Node project with artifacts
    let node_proj = root.join("my_node_app");
    fs::create_dir_all(&node_proj).unwrap();
    fs::write(node_proj.join("package.json"), "{}").unwrap();
    fs::create_dir_all(node_proj.join("node_modules/express")).unwrap();
    fs::write(node_proj.join("node_modules/express/index.js"), "code").unwrap();

    // Non-project directory
    let random = root.join("documents");
    fs::create_dir_all(&random).unwrap();
    fs::write(random.join("notes.txt"), "notes").unwrap();

    let projects = scan_directory(&root, None).unwrap();

    assert_eq!(projects.len(), 2);
    let kinds: Vec<ProjectKind> = projects.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&ProjectKind::Rust));
    assert!(kinds.contains(&ProjectKind::Node));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scan_respects_max_depth() {
    let root = test_dir("scan_depth");

    // Shallow project (depth 1)
    let shallow = root.join("shallow");
    fs::create_dir_all(&shallow).unwrap();
    fs::write(shallow.join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir_all(shallow.join("target")).unwrap();
    fs::write(shallow.join("target/bin"), "data").unwrap();

    // Deep project (depth 3)
    let deep = root.join("a/b/deep");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("package.json"), "{}").unwrap();
    fs::create_dir_all(deep.join("node_modules")).unwrap();
    fs::write(deep.join("node_modules/mod.js"), "code").unwrap();

    let projects = scan_directory(&root, Some(2)).unwrap();

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].kind, ProjectKind::Rust);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scan_empty_directory() {
    let root = test_dir("scan_empty");
    let projects = scan_directory(&root, None).unwrap();
    assert!(projects.is_empty());
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn scan_excludes_projects_with_no_artifacts() {
    let root = test_dir("scan_no_artifacts");

    let proj = root.join("clean_project");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("Cargo.toml"), "[package]").unwrap();
    // No target/ directory

    let projects = scan_directory(&root, None).unwrap();
    assert!(projects.is_empty()); // 0 cleanable bytes → filtered out
    fs::remove_dir_all(&root).unwrap();
}
