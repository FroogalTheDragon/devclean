use std::fmt;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// The kind of development project detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectKind {
    Rust,
    Node,
    Python,
    Java,
    DotNet,
    Go,
    Zig,
    CMake,
    Swift,
    Elixir,
    Haskell,
    Dart,
    Ruby,
    Scala,
    Unity,
    Godot,
    Terraform,
}

impl ProjectKind {
    /// Returns the marker file(s) used to detect this project kind.
    pub fn marker_files(&self) -> &[&str] {
        match self {
            Self::Rust => &["Cargo.toml"],
            Self::Node => &["package.json"],
            Self::Python => &["pyproject.toml", "setup.py", "requirements.txt"],
            Self::Java => &["pom.xml", "build.gradle", "build.gradle.kts"],
            Self::DotNet => &["*.csproj", "*.fsproj", "*.sln"],
            Self::Go => &["go.mod"],
            Self::Zig => &["build.zig"],
            Self::CMake => &["CMakeLists.txt"],
            Self::Swift => &["Package.swift"],
            Self::Elixir => &["mix.exs"],
            Self::Haskell => &["stack.yaml", "*.cabal"],
            Self::Dart => &["pubspec.yaml"],
            Self::Ruby => &["Gemfile"],
            Self::Scala => &["build.sbt"],
            Self::Unity => &["ProjectSettings/ProjectVersion.txt"],
            Self::Godot => &["project.godot"],
            Self::Terraform => &["main.tf", "*.tf"],
        }
    }

    /// Returns the directories that can be safely cleaned for this project kind.
    pub fn cleanable_dirs(&self) -> &[&str] {
        match self {
            Self::Rust => &["target"],
            Self::Node => &["node_modules", ".next", ".nuxt", "dist", ".cache"],
            Self::Python => &["__pycache__", ".venv", "venv", ".tox", "*.egg-info", ".mypy_cache", ".pytest_cache"],
            Self::Java => &["target", "build", ".gradle"],
            Self::DotNet => &["bin", "obj"],
            Self::Go => &[],  // Go modules are shared, not per-project artifacts
            Self::Zig => &["zig-cache", "zig-out"],
            Self::CMake => &["build", "cmake-build-debug", "cmake-build-release"],
            Self::Swift => &[".build"],
            Self::Elixir => &["_build", "deps"],
            Self::Haskell => &[".stack-work"],
            Self::Dart => &[".dart_tool", "build"],
            Self::Ruby => &["vendor/bundle"],
            Self::Scala => &["target", "project/target"],
            Self::Unity => &["Library", "Temp", "Obj", "Logs"],
            Self::Godot => &[".godot"],
            Self::Terraform => &[".terraform"],
        }
    }

    /// Returns all known project kinds.
    pub fn all() -> &'static [ProjectKind] {
        &[
            Self::Rust,
            Self::Node,
            Self::Python,
            Self::Java,
            Self::DotNet,
            Self::Go,
            Self::Zig,
            Self::CMake,
            Self::Swift,
            Self::Elixir,
            Self::Haskell,
            Self::Dart,
            Self::Ruby,
            Self::Scala,
            Self::Unity,
            Self::Godot,
            Self::Terraform,
        ]
    }
}

impl fmt::Display for ProjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Rust => "Rust",
            Self::Node => "Node.js",
            Self::Python => "Python",
            Self::Java => "Java",
            Self::DotNet => ".NET",
            Self::Go => "Go",
            Self::Zig => "Zig",
            Self::CMake => "CMake",
            Self::Swift => "Swift",
            Self::Elixir => "Elixir",
            Self::Haskell => "Haskell",
            Self::Dart => "Dart",
            Self::Ruby => "Ruby",
            Self::Scala => "Scala",
            Self::Unity => "Unity",
            Self::Godot => "Godot",
            Self::Terraform => "Terraform",
        };
        write!(f, "{name}")
    }
}

/// A directory within a project that can be cleaned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanTarget {
    /// Absolute path to the cleanable directory.
    pub path: PathBuf,
    /// Display name (e.g. "node_modules", "target").
    pub name: String,
    /// Size in bytes.
    pub size_bytes: u64,
}

/// A discovered developer project on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedProject {
    /// The project root directory.
    pub path: PathBuf,
    /// The detected project kind.
    pub kind: ProjectKind,
    /// A human-friendly project name (usually the directory name).
    pub name: String,
    /// When the project was last modified (based on marker file).
    pub last_modified: DateTime<Local>,
    /// Directories that can be cleaned.
    pub clean_targets: Vec<CleanTarget>,
    /// Total reclaimable bytes across all clean targets.
    pub total_cleanable_bytes: u64,
}
