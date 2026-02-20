use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Local};
use rayon::prelude::*;
use walkdir::WalkDir;

use super::project::{CleanTarget, ProjectKind, ScannedProject};

/// Directories to skip during scanning (to avoid infinite loops or irrelevant results).
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    ".gradle",
    "Library", // Unity
    ".terraform",
    ".godot",
    ".stack-work",
    ".build",
    "zig-cache",
    "zig-out",
];

/// A simple spinner for terminal feedback.
struct Spinner {
    frames: &'static [&'static str],
    idx: usize,
}

impl Spinner {
    fn new() -> Self {
        Self {
            frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            idx: 0,
        }
    }

    fn tick(&mut self, msg: &str) {
        let frame = self.frames[self.idx % self.frames.len()];
        eprint!("\r  \x1b[36m{frame}\x1b[0m {msg}");
        let _ = io::stderr().flush();
        self.idx += 1;
    }

    fn finish(&self) {
        // Clear the spinner line
        eprint!("\r\x1b[2K");
        let _ = io::stderr().flush();
    }
}

/// Scan a directory tree for developer projects.
///
/// Returns a list of discovered projects with their cleanable targets and sizes.
pub fn scan_directory(root: &Path, max_depth: Option<usize>) -> Result<Vec<ScannedProject>> {
    let mut spinner = Spinner::new();
    spinner.tick(&format!("Scanning {}...", root.display()));

    // Phase 1: Collect candidate project roots
    let candidates = find_project_roots(root, max_depth, &mut spinner)?;

    spinner.tick(&format!(
        "Found {} projects, calculating sizes...",
        candidates.len()
    ));

    // Phase 2: Analyze each project in parallel
    let projects: Vec<ScannedProject> = candidates
        .into_par_iter()
        .filter_map(|(path, kind)| analyze_project(&path, kind).ok())
        .filter(|p| p.total_cleanable_bytes > 0)
        .collect();

    spinner.finish();

    Ok(projects)
}

/// Walk the filesystem to find project root directories.
fn find_project_roots(
    root: &Path,
    max_depth: Option<usize>,
    spinner: &mut Spinner,
) -> Result<Vec<(PathBuf, ProjectKind)>> {
    let mut candidates = Vec::new();
    let mut walker = WalkDir::new(root).follow_links(false);

    if let Some(depth) = max_depth {
        walker = walker.max_depth(depth);
    }

    let mut dirs_scanned: u64 = 0;

    for entry in walker.into_iter().filter_entry(|e| should_visit(e)) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        dirs_scanned += 1;
        if dirs_scanned % 200 == 0 {
            spinner.tick(&format!("Scanning... {} directories checked", dirs_scanned));
        }

        let dir_path = entry.path();

        // Check if this directory is a project root
        if let Some(kind) = detect_project_kind(dir_path) {
            candidates.push((dir_path.to_path_buf(), kind));
        }
    }

    Ok(candidates)
}

/// Determine if a walkdir entry should be descended into.
fn should_visit(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }

    let name = entry.file_name().to_string_lossy();

    // Skip hidden directories (except the root)
    if name.starts_with('.') && entry.depth() > 0 {
        // Allow descending into directories not in skip list
        return !SKIP_DIRS.contains(&name.as_ref());
    }

    // Skip known artifact directories
    !SKIP_DIRS.contains(&name.as_ref())
}

/// Detect what kind of project a directory contains, if any.
fn detect_project_kind(dir: &Path) -> Option<ProjectKind> {
    for kind in ProjectKind::all() {
        for marker in kind.marker_files() {
            if marker.contains('*') {
                // Glob pattern - check if any file matches
                if let Ok(entries) = fs::read_dir(dir) {
                    let suffix = marker.trim_start_matches('*');
                    for entry in entries.flatten() {
                        if entry.file_name().to_string_lossy().ends_with(suffix) {
                            return Some(*kind);
                        }
                    }
                }
            } else if marker.contains('/') {
                // Path with subdirectory (e.g. Unity)
                if dir.join(marker).exists() {
                    return Some(*kind);
                }
            } else if dir.join(marker).is_file() {
                return Some(*kind);
            }
        }
    }
    None
}

/// Analyze a single project: find cleanable targets and calculate sizes.
fn analyze_project(project_root: &Path, kind: ProjectKind) -> Result<ScannedProject> {
    let name = project_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| project_root.display().to_string());

    let last_modified = get_last_modified(project_root, &kind)?;

    let mut clean_targets = Vec::new();

    for dir_pattern in kind.cleanable_dirs() {
        if dir_pattern.contains('*') {
            // Glob-style matching within the project root
            let suffix = dir_pattern.trim_start_matches('*');
            if let Ok(entries) = fs::read_dir(project_root) {
                for entry in entries.flatten() {
                    let entry_name = entry.file_name().to_string_lossy().to_string();
                    if entry_name.ends_with(suffix) && entry.path().is_dir() {
                        if let Ok(size) = dir_size(&entry.path()) {
                            if size > 0 {
                                clean_targets.push(CleanTarget {
                                    path: entry.path(),
                                    name: entry_name,
                                    size_bytes: size,
                                });
                            }
                        }
                    }
                }
            }
        } else if dir_pattern.contains('/') {
            // Nested path
            let target = project_root.join(dir_pattern);
            if target.is_dir() {
                if let Ok(size) = dir_size(&target) {
                    if size > 0 {
                        clean_targets.push(CleanTarget {
                            path: target,
                            name: dir_pattern.to_string(),
                            size_bytes: size,
                        });
                    }
                }
            }
        } else {
            let target = project_root.join(dir_pattern);
            if target.is_dir() {
                if let Ok(size) = dir_size(&target) {
                    if size > 0 {
                        clean_targets.push(CleanTarget {
                            path: target,
                            name: dir_pattern.to_string(),
                            size_bytes: size,
                        });
                    }
                }
            }
        }

        // Also find __pycache__ dirs recursively for Python projects
        if kind == ProjectKind::Python && *dir_pattern == "__pycache__" {
            find_pycache_recursive(project_root, &mut clean_targets);
        }
    }

    let total_cleanable_bytes = clean_targets.iter().map(|t| t.size_bytes).sum();

    Ok(ScannedProject {
        path: project_root.to_path_buf(),
        kind,
        name,
        last_modified,
        clean_targets,
        total_cleanable_bytes,
    })
}

/// Get the last modified time of a project based on its marker files.
fn get_last_modified(project_root: &Path, kind: &ProjectKind) -> Result<DateTime<Local>> {
    let mut latest: Option<SystemTime> = None;

    for marker in kind.marker_files() {
        if marker.contains('*') || marker.contains('/') {
            continue;
        }
        let marker_path = project_root.join(marker);
        if let Ok(meta) = fs::metadata(&marker_path) {
            if let Ok(modified) = meta.modified() {
                latest = Some(match latest {
                    Some(prev) if modified > prev => modified,
                    Some(prev) => prev,
                    None => modified,
                });
            }
        }
    }

    // Fallback to the project directory's own mtime
    let time = match latest {
        Some(t) => t,
        None => fs::metadata(project_root)?.modified()?,
    };

    Ok(DateTime::<Local>::from(time))
}

/// Calculate the total size of a directory recursively.
fn dir_size(path: &Path) -> Result<u64> {
    let mut total: u64 = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }

    Ok(total)
}

/// Recursively find all __pycache__ directories under a path.
fn find_pycache_recursive(root: &Path, targets: &mut Vec<CleanTarget>) {
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Don't descend into other artifact dirs
            !SKIP_DIRS.contains(&name.as_ref()) || name == "__pycache__"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir()
            && entry.file_name() == "__pycache__"
            && entry.depth() > 0 // Skip the root-level one, already handled
        {
            if let Ok(size) = dir_size(entry.path()) {
                if size > 0 {
                    let relative = entry.path().strip_prefix(root).unwrap_or(entry.path());
                    targets.push(CleanTarget {
                        path: entry.path().to_path_buf(),
                        name: relative.display().to_string(),
                        size_bytes: size,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_rust_project() {
        let dir = std::env::temp_dir().join("devclean_test_rust");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Rust));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_node_project() {
        let dir = std::env::temp_dir().join("devclean_test_node");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("package.json"), "{}").unwrap();
        assert_eq!(detect_project_kind(&dir), Some(ProjectKind::Node));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_detect_no_project() {
        let dir = std::env::temp_dir().join("devclean_test_empty");
        let _ = fs::create_dir_all(&dir);
        assert_eq!(detect_project_kind(&dir), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_dir_size() {
        let dir = std::env::temp_dir().join("devclean_test_size");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("file1.txt"), "hello").unwrap(); // 5 bytes
        fs::write(dir.join("file2.txt"), "world!").unwrap(); // 6 bytes
        let size = dir_size(&dir).unwrap();
        assert_eq!(size, 11);
        let _ = fs::remove_dir_all(&dir);
    }
}
