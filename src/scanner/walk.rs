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
pub const SKIP_DIRS: &[&str] = &[
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

    let candidates = find_project_roots(root, max_depth, &mut spinner)?;

    spinner.tick(&format!(
        "Found {} projects, calculating sizes...",
        candidates.len()
    ));

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

    for entry in walker.into_iter().filter_entry(should_visit) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        dirs_scanned += 1;
        #[allow(clippy::manual_is_multiple_of)]
        if dirs_scanned % 200 == 0 {
            spinner.tick(&format!("Scanning... {} directories checked", dirs_scanned));
        }

        let dir_path = entry.path();

        if let Some(kind) = detect_project_kind(dir_path) {
            candidates.push((dir_path.to_path_buf(), kind));
        }
    }

    Ok(candidates)
}

/// Determine if a walkdir entry should be descended into.
///
/// Skips all hidden directories (dot-prefixed) at depth > 0, as well as
/// any directory in [`SKIP_DIRS`] (build artifacts, dependency caches, etc.).
pub fn should_visit(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }

    let name = entry.file_name().to_string_lossy();

    // Skip all hidden (dot-prefixed) directories below the root — these are
    // almost never useful to scan (.cache, .local, .backup, etc.) and the
    // known artifact dirs (.git, .venv, …) are already in SKIP_DIRS.
    if name.starts_with('.') && entry.depth() > 0 {
        return false;
    }

    !SKIP_DIRS.contains(&name.as_ref())
}

/// Detect what kind of project a directory contains, if any.
pub fn detect_project_kind(dir: &Path) -> Option<ProjectKind> {
    for kind in ProjectKind::all() {
        for marker in kind.marker_files() {
            if marker.contains('*') {
                if let Ok(entries) = fs::read_dir(dir) {
                    let suffix = marker.trim_start_matches('*');
                    for entry in entries.flatten() {
                        if entry.file_name().to_string_lossy().ends_with(suffix) {
                            return Some(*kind);
                        }
                    }
                }
            } else if marker.contains('/') {
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
pub fn analyze_project(project_root: &Path, kind: ProjectKind) -> Result<ScannedProject> {
    let name = project_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| project_root.display().to_string());

    let last_modified = get_last_modified(project_root, &kind)?;

    let mut clean_targets: Vec<CleanTarget> = kind
        .cleanable_dirs()
        .iter()
        .flat_map(|pattern| resolve_pattern(project_root, pattern))
        .filter_map(|(path, name)| as_clean_target(path, name))
        .collect();

    if kind == ProjectKind::Python {
        find_pycache_recursive(project_root, &mut clean_targets);
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

/// Resolve a cleanable-dir pattern into concrete (path, display_name) candidates.
///
/// - `"*suffix"` → glob: scan the project root for matching directories
/// - `"sub/dir"` → nested path: check if the exact subdirectory exists
/// - `"dirname"` → simple: check if the directory exists at the project root
fn resolve_pattern(project_root: &Path, pattern: &str) -> Vec<(PathBuf, String)> {
    if pattern.contains('*') {
        // Glob pattern — match directory names by suffix
        let suffix = pattern.trim_start_matches('*');
        fs::read_dir(project_root)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.ends_with(suffix) && e.path().is_dir()
            })
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                (e.path(), name)
            })
            .collect()
    } else {
        // Exact path (simple name or nested like "project/target")
        let target = project_root.join(pattern);
        if target.is_dir() {
            vec![(target, pattern.to_string())]
        } else {
            vec![]
        }
    }
}

/// Try to turn a candidate directory into a CleanTarget. Returns None if empty or unreadable.
fn as_clean_target(path: PathBuf, name: String) -> Option<CleanTarget> {
    let size = dir_size(&path).ok()?;
    (size > 0).then_some(CleanTarget {
        path,
        name,
        size_bytes: size,
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
        if let Ok(meta) = fs::metadata(&marker_path)
            && let Ok(modified) = meta.modified()
        {
            latest = Some(match latest {
                Some(prev) if modified > prev => modified,
                Some(prev) => prev,
                None => modified,
            });
        }
    }

    let time = match latest {
        Some(t) => t,
        None => fs::metadata(project_root)?.modified()?,
    };

    Ok(DateTime::<Local>::from(time))
}

/// Calculate the total size of a directory recursively.
pub fn dir_size(path: &Path) -> Result<u64> {
    let mut total: u64 = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && let Ok(meta) = entry.metadata()
        {
            total += meta.len();
        }
    }

    Ok(total)
}

/// Recursively find all __pycache__ directories under a path.
pub fn find_pycache_recursive(root: &Path, targets: &mut Vec<CleanTarget>) {
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.contains(&name.as_ref()) || name == "__pycache__"
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir()
            && entry.file_name() == "__pycache__"
            && entry.depth() > 0
            && let Ok(size) = dir_size(entry.path())
            && size > 0
        {
            let relative = entry.path().strip_prefix(root).unwrap_or(entry.path());
            targets.push(CleanTarget {
                path: entry.path().to_path_buf(),
                name: relative.display().to_string(),
                size_bytes: size,
            });
        }
    }
}
