use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::scanner::ScannedProject;

/// Result of a clean operation on a single project.
#[derive(Debug)]
pub struct CleanResult {
    pub project_name: String,
    pub targets_cleaned: usize,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

/// Clean the specified targets from a project.
///
/// If `dry_run` is true, only reports what *would* be cleaned without deleting anything.
pub fn clean_project(project: &ScannedProject, dry_run: bool) -> Result<CleanResult> {
    let mut result = CleanResult {
        project_name: project.name.clone(),
        targets_cleaned: 0,
        bytes_freed: 0,
        errors: Vec::new(),
    };

    for target in &project.clean_targets {
        if dry_run {
            result.targets_cleaned += 1;
            result.bytes_freed += target.size_bytes;
            continue;
        }

        match remove_dir_all(&target.path) {
            Ok(()) => {
                result.targets_cleaned += 1;
                result.bytes_freed += target.size_bytes;
            }
            Err(e) => {
                result.errors.push(format!(
                    "Failed to remove {}: {}",
                    target.path.display(),
                    e
                ));
            }
        }
    }

    Ok(result)
}

/// Remove a directory and all its contents.
///
/// This is a wrapper around `fs::remove_dir_all` with better error context.
fn remove_dir_all(path: &Path) -> Result<()> {
    fs::remove_dir_all(path)
        .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
    Ok(())
}

/// Clean multiple projects and return results.
pub fn clean_projects(
    projects: &[&ScannedProject],
    dry_run: bool,
) -> Vec<CleanResult> {
    projects
        .iter()
        .map(|p| {
            clean_project(p, dry_run).unwrap_or_else(|e| CleanResult {
                project_name: p.name.clone(),
                targets_cleaned: 0,
                bytes_freed: 0,
                errors: vec![e.to_string()],
            })
        })
        .collect()
}
