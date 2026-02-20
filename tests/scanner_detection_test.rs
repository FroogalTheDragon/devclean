//! Tests for project kind detection across all 17 supported project types.

use std::fs;
use std::path::PathBuf;

use dev_sweep::scanner::ProjectKind;
use dev_sweep::scanner::walk::detect_project_kind;

/// Helper: create a fresh temp dir for a test.
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dev_sweep_test_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ── Simple marker files ─────────────────────────────────────────────────────

#[test]
fn detect_rust() {
    let dir = test_dir("detect_rust");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Rust));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_node() {
    let dir = test_dir("detect_node");
    fs::write(dir.join("package.json"), "{}").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Node));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_python_pyproject() {
    let dir = test_dir("detect_py_pyproject");
    fs::write(dir.join("pyproject.toml"), "[project]").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Python));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_python_setup_py() {
    let dir = test_dir("detect_py_setup");
    fs::write(dir.join("setup.py"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Python));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_python_requirements() {
    let dir = test_dir("detect_py_reqs");
    fs::write(dir.join("requirements.txt"), "flask").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Python));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_java_maven() {
    let dir = test_dir("detect_java_mvn");
    fs::write(dir.join("pom.xml"), "<project>").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Java));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_java_gradle() {
    let dir = test_dir("detect_java_gradle");
    fs::write(dir.join("build.gradle"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Java));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_java_gradle_kts() {
    let dir = test_dir("detect_java_gradle_kts");
    fs::write(dir.join("build.gradle.kts"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Java));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_go() {
    let dir = test_dir("detect_go");
    fs::write(dir.join("go.mod"), "module example").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Go));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_zig() {
    let dir = test_dir("detect_zig");
    fs::write(dir.join("build.zig"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Zig));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_cmake() {
    let dir = test_dir("detect_cmake");
    fs::write(dir.join("CMakeLists.txt"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::CMake));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_swift() {
    let dir = test_dir("detect_swift");
    fs::write(dir.join("Package.swift"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Swift));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_elixir() {
    let dir = test_dir("detect_elixir");
    fs::write(dir.join("mix.exs"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Elixir));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_dart() {
    let dir = test_dir("detect_dart");
    fs::write(dir.join("pubspec.yaml"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Dart));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_ruby() {
    let dir = test_dir("detect_ruby");
    fs::write(dir.join("Gemfile"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Ruby));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_scala() {
    let dir = test_dir("detect_scala");
    fs::write(dir.join("build.sbt"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Scala));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_godot() {
    let dir = test_dir("detect_godot");
    fs::write(dir.join("project.godot"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Godot));
    fs::remove_dir_all(&dir).unwrap();
}

// ── Glob-based marker files ─────────────────────────────────────────────────

#[test]
fn detect_dotnet_csproj() {
    let dir = test_dir("detect_dotnet_cs");
    fs::write(dir.join("MyApp.csproj"), "<Project>").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::DotNet));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_dotnet_fsproj() {
    let dir = test_dir("detect_dotnet_fs");
    fs::write(dir.join("MyApp.fsproj"), "<Project>").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::DotNet));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_dotnet_sln() {
    let dir = test_dir("detect_dotnet_sln");
    fs::write(dir.join("MyApp.sln"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::DotNet));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_haskell_stack() {
    let dir = test_dir("detect_haskell_stack");
    fs::write(dir.join("stack.yaml"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Haskell));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_haskell_cabal_glob() {
    let dir = test_dir("detect_haskell_cabal");
    fs::write(dir.join("myproject.cabal"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Haskell));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_terraform_main() {
    let dir = test_dir("detect_tf_main");
    fs::write(dir.join("main.tf"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Terraform));
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_terraform_glob() {
    let dir = test_dir("detect_tf_glob");
    fs::write(dir.join("vpc.tf"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Terraform));
    fs::remove_dir_all(&dir).unwrap();
}

// ── Subdirectory-based marker files ─────────────────────────────────────────

#[test]
fn detect_unity() {
    let dir = test_dir("detect_unity");
    fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
    fs::write(dir.join("ProjectSettings/ProjectVersion.txt"), "").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Unity));
    fs::remove_dir_all(&dir).unwrap();
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn detect_empty_directory() {
    let dir = test_dir("detect_empty");
    assert_eq!(detect_project_kind(&dir), None);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_directory_with_random_files() {
    let dir = test_dir("detect_random");
    fs::write(dir.join("readme.md"), "# Hello").unwrap();
    fs::write(dir.join("notes.txt"), "stuff").unwrap();
    assert_eq!(detect_project_kind(&dir), None);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn detect_priority_rust_over_node() {
    // Rust appears first in ProjectKind::all(), so it should win
    let dir = test_dir("detect_priority");
    fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    fs::write(dir.join("package.json"), "{}").unwrap();
    assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Rust));
    fs::remove_dir_all(&dir).unwrap();
}
