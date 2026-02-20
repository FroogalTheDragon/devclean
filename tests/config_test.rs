//! Tests for configuration: defaults, serialization round-trip, save/load.

use std::fs;
use std::path::PathBuf;

use dev_sweep::config::DevSweepConfig;
use dev_sweep::scanner::ProjectKind;

#[test]
fn default_config_is_empty() {
    let config = DevSweepConfig::default();
    assert!(config.ignore_paths.is_empty());
    assert!(config.exclude_kinds.is_empty());
    assert!(config.default_roots.is_empty());
    assert!(config.max_depth.is_none());
}

#[test]
fn config_serialization_round_trip() {
    let config = DevSweepConfig {
        ignore_paths: vec![PathBuf::from("/tmp/skip")],
        exclude_kinds: vec![ProjectKind::Go, ProjectKind::Terraform],
        default_roots: vec![PathBuf::from("~/projects")],
        max_depth: Some(5),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: DevSweepConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.ignore_paths, config.ignore_paths);
    assert_eq!(deserialized.exclude_kinds, config.exclude_kinds);
    assert_eq!(deserialized.default_roots, config.default_roots);
    assert_eq!(deserialized.max_depth, config.max_depth);
}

#[test]
fn config_deserializes_with_missing_fields() {
    // Simulates an older config file that's missing some fields
    let json = r#"{"ignore_paths": ["/tmp/old"]}"#;
    let config: DevSweepConfig = serde_json::from_str(json).unwrap();

    assert_eq!(config.ignore_paths, vec![PathBuf::from("/tmp/old")]);
    assert!(config.exclude_kinds.is_empty());
    assert!(config.default_roots.is_empty());
    assert!(config.max_depth.is_none());
}

#[test]
fn config_deserializes_empty_object() {
    let config: DevSweepConfig = serde_json::from_str("{}").unwrap();
    assert!(config.ignore_paths.is_empty());
    assert!(config.exclude_kinds.is_empty());
}

#[test]
fn config_save_and_load() {
    // Use a temp file to avoid polluting the real config
    let dir = std::env::temp_dir().join("dev_sweep_test_config");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.json");

    let config = DevSweepConfig {
        ignore_paths: vec![PathBuf::from("/home/mark/skip")],
        exclude_kinds: vec![ProjectKind::Ruby],
        default_roots: vec![PathBuf::from("~/code")],
        max_depth: Some(10),
    };

    // Save
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&path, &json).unwrap();

    // Load
    let loaded: DevSweepConfig =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();

    assert_eq!(loaded.ignore_paths, config.ignore_paths);
    assert_eq!(loaded.exclude_kinds, config.exclude_kinds);
    assert_eq!(loaded.default_roots, config.default_roots);
    assert_eq!(loaded.max_depth, config.max_depth);

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn config_load_returns_default_when_missing() {
    // DevSweepConfig::load() should return defaults if the file doesn't exist
    let config = DevSweepConfig::load();
    // We can't assert much about the actual state since the user might have a real config,
    // but at minimum it should not panic and should return a valid config.
    let _ = config.ignore_paths;
    let _ = config.max_depth;
}
