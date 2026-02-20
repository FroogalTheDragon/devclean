# 🧹 dev-sweep

A fast, interactive CLI tool to find and clean build artifacts and dependency caches across all your developer projects.

Every developer accumulates gigabytes of `node_modules/`, `target/`, `.venv/`, and `build/` directories across dozens of old projects they haven't touched in months. **dev-sweep** scans your filesystem, finds those space hogs, and lets you reclaim disk space in seconds.

## Features

- **Smart project detection** — automatically identifies 17 project types by their marker files
- **Parallel scanning** — uses [rayon](https://crates.io/crates/rayon) for concurrent filesystem traversal and size calculation
- **Interactive cleaning** — select individual projects by number, range (`3-7`), or `all`
- **Safe by default** — confirmation prompts before every destructive operation; `--dry-run` to preview
- **Age filtering** — target stale projects with `--older-than 3m`
- **JSON output** — machine-readable mode (`--json`) for scripting and pipelines
- **Beautiful terminal output** — colored Unicode tables, animated spinner, human-readable sizes
- **Persistent configuration** — save ignored paths, excluded project types, and default scan roots
- **Minimal dependencies** — only 7 crates; ANSI colors and table rendering implemented from scratch
- **Comprehensive test suite** — 137 tests across 7 test files covering every module

## Installation

### From crates.io

```bash
cargo install dev-sweep
```

### From source

Requires [Rust](https://www.rust-lang.org/tools/install) 1.85+ (edition 2024).

```bash
git clone https://github.com/markwaidjr/dev-sweep.git
cd dev-sweep
cargo install --path .
```

### Build locally (without installing)

```bash
cargo build --release
# Binary: ./target/release/dev-sweep
```

## Usage

### Scan (default command)

Discover projects and display reclaimable space:

```bash
# Scan the current directory
dev-sweep

# Scan a specific directory
dev-sweep ~/projects

# Limit scan depth
dev-sweep ~/projects -d 3

# Only show projects untouched for 3+ months
dev-sweep --older-than 3m ~/projects

# Output as JSON
dev-sweep --json ~/projects
```

### Clean

Interactively select and remove build artifacts:

```bash
# Interactive mode — pick which projects to clean
dev-sweep clean ~/projects

# Preview what would be cleaned (no deletions)
dev-sweep clean --dry-run ~/projects

# Clean everything without prompting
dev-sweep clean --all ~/projects

# Clean only stale projects
dev-sweep clean --older-than 6m ~/projects
```

When running interactively, `dev-sweep clean` presents a numbered list and accepts:

- Single numbers: `3`
- Comma-separated: `1,4,7`
- Ranges: `3-8`
- Mixed: `1,3-5,9`
- Everything: `all`

### Summary

Quick overview grouped by project type:

```bash
dev-sweep summary ~/projects
```

```
  📊 dev-sweep summary for /home/mark/projects

  Total projects:     28
  Reclaimable space:  53.4 GB

  By project type:
            Rust  22 projects, 48.1 GB
         Node.js  4 projects, 4.6 GB
          Python  1 projects, 33.0 MB
            .NET  1 projects, 695.2 MB
```

### Config

Manage persistent settings stored at `~/.config/dev-sweep/config.json`:

```bash
# Show current configuration
dev-sweep config --show

# Reset to defaults
dev-sweep config --reset
```

## CLI Reference

```
Usage: dev-sweep [OPTIONS] [PATH] [COMMAND]

Commands:
  scan      Scan for projects and show what can be cleaned (default)
  clean     Interactively select and clean projects
  summary   Show a quick summary of reclaimable space
  config    Manage dev-sweep configuration
  help      Print help for a command

Arguments:
  [PATH]    Directory to scan (defaults to current directory)

Options:
  -d, --max-depth <N>            Maximum directory depth to scan
  -o, --older-than <AGE>         Only show projects older than this (e.g. "30d", "3m", "1y")
      --json                     Output results as JSON
  -h, --help                     Print help
  -V, --version                  Print version
```

**`clean` subcommand options:**

```
  -a, --all       Clean all found projects without prompting
      --dry-run   Show what would be cleaned without actually deleting
```

### Age format

The `--older-than` flag accepts a number followed by a unit:

| Unit | Meaning          | Example |
|------|------------------|---------|
| `d`  | Days             | `30d`   |
| `w`  | Weeks            | `4w`    |
| `m`  | Months (30 days) | `3m`    |
| `y`  | Years (365 days) | `1y`    |

## Supported Project Types

| Type | Marker Files | Cleaned Directories |
|---|---|---|
| **Rust** | `Cargo.toml` | `target/` |
| **Node.js** | `package.json` | `node_modules/`, `.next/`, `.nuxt/`, `dist/`, `.cache/` |
| **Python** | `pyproject.toml`, `setup.py`, `requirements.txt` | `__pycache__/` (recursive), `.venv/`, `venv/`, `.tox/`, `*.egg-info/`, `.mypy_cache/`, `.pytest_cache/` |
| **Java** | `pom.xml`, `build.gradle`, `build.gradle.kts` | `target/`, `build/`, `.gradle/` |
| **.NET** | `*.csproj`, `*.fsproj`, `*.sln` | `bin/`, `obj/` |
| **Go** | `go.mod` | *(detected but no per-project artifacts to clean)* |
| **Zig** | `build.zig` | `zig-cache/`, `zig-out/` |
| **CMake** | `CMakeLists.txt` | `build/`, `cmake-build-debug/`, `cmake-build-release/` |
| **Swift** | `Package.swift` | `.build/` |
| **Elixir** | `mix.exs` | `_build/`, `deps/` |
| **Haskell** | `stack.yaml`, `*.cabal` | `.stack-work/` |
| **Dart** | `pubspec.yaml` | `.dart_tool/`, `build/` |
| **Ruby** | `Gemfile` | `vendor/bundle/` |
| **Scala** | `build.sbt` | `target/`, `project/target/` |
| **Unity** | `ProjectSettings/ProjectVersion.txt` | `Library/`, `Temp/`, `Obj/`, `Logs/` |
| **Godot** | `project.godot` | `.godot/` |
| **Terraform** | `main.tf`, `*.tf` | `.terraform/` |

Marker files support three matching strategies:
- **Exact name** — `Cargo.toml`, `package.json`
- **Glob suffix** — `*.csproj`, `*.cabal`, `*.tf`
- **Nested path** — `ProjectSettings/ProjectVersion.txt`

## Configuration

dev-sweep looks for a config file at `~/.config/dev-sweep/config.json`. All fields are optional and default to empty/null:

```json
{
  "ignore_paths": ["/home/mark/projects/keep-this"],
  "exclude_kinds": ["Go", "Terraform"],
  "default_roots": ["~/projects", "~/work"],
  "max_depth": 5
}
```

| Field | Type | Description |
|---|---|---|
| `ignore_paths` | `string[]` | Absolute paths to skip during scanning |
| `exclude_kinds` | `string[]` | Project types to exclude (e.g. `"Rust"`, `"Node"`, `"Python"`) |
| `default_roots` | `string[]` | Default directories to scan when no path is given |
| `max_depth` | `number \| null` | Maximum directory traversal depth |

## Project Structure

```
dev-sweep/
├── Cargo.toml                          # Package manifest and dependencies
├── Cargo.lock                          # Locked dependency versions
├── LICENSE                             # MIT license
├── README.md
│
├── src/
│   ├── lib.rs                          # Library root — re-exports all modules
│   ├── main.rs                         # CLI entry point (clap), commands, arg parsing
│   ├── util.rs                         # Pure utilities: parse_age, format_bytes,
│   │                                   #   visible_len, pad_left/right, format_age,
│   │                                   #   truncate, shorten_path
│   ├── scanner/
│   │   ├── mod.rs                      # Re-exports
│   │   ├── project.rs                  # ProjectKind enum (17 variants), marker files,
│   │   │                               #   cleanable dirs, CleanTarget, ScannedProject
│   │   └── walk.rs                     # Filesystem walker, project detection,
│   │                                   #   analyze_project, dir_size, resolve_pattern,
│   │                                   #   pycache discovery, skip-dir filtering
│   ├── cleaner/
│   │   └── mod.rs                      # clean_project (with dry-run), clean_projects,
│   │                                   #   CleanResult, safe rm -rf wrapper
│   ├── config/
│   │   └── mod.rs                      # DevSweepConfig: load/save JSON, defaults
│   └── tui/
│       ├── mod.rs                      # Re-exports
│       └── display.rs                  # ANSI color helpers, Unicode table renderer,
│                                       #   print_results_table, print_clean_summary,
│                                       #   multi_select prompt, parse_selection, confirm
│
└── tests/
    ├── age_parser_test.rs              # 17 tests — parse_age valid/invalid inputs
    ├── cleaner_test.rs                 #  6 tests — dry-run, deletion, errors, multi-project
    ├── config_test.rs                  #  6 tests — defaults, round-trip, partial JSON, save/load
    ├── display_test.rs                 # 44 tests — format_bytes, visible_len, pad_*, format_age,
    │                                   #             truncate, shorten_path, ANSI helpers
    ├── scanner_analysis_test.rs        # 20 tests — dir_size, should_visit, analyze_project,
    │                                   #             pycache discovery, scan_directory integration
    ├── scanner_detection_test.rs       # 28 tests — all 17 project types, globs, subdirs, edge cases
    └── selection_parser_test.rs        # 16 tests — numbers, ranges, commas, dedup, error cases
                                        # ─────────
                                        # 137 total
```

### How scanning works

1. **Walk** — `walkdir` traverses the directory tree, skipping known artifact directories (`.git`, `node_modules`, `target`, etc.) to avoid descending into millions of files.
2. **Detect** — Each directory is checked against the marker files for all 17 project types. The first match wins (ordered by `ProjectKind::all()`).
3. **Analyze** — For each detected project, `resolve_pattern()` maps cleanable-dir patterns to concrete directory paths, and `as_clean_target()` calculates the size of each. Python projects additionally run `find_pycache_recursive()` to discover nested `__pycache__/` directories.
4. **Filter** — Projects with zero reclaimable bytes are excluded. The results are sorted by size (largest first) and optionally filtered by age.
5. **Display** — Results are rendered as a Unicode box-drawing table with ANSI colors, or as JSON.

Size calculation and project analysis run in parallel using `rayon::par_iter`.

## Dependencies

| Crate | Purpose |
|---|---|
| [clap](https://crates.io/crates/clap) | Command-line argument parsing with derive macros |
| [walkdir](https://crates.io/crates/walkdir) | Recursive directory traversal |
| [rayon](https://crates.io/crates/rayon) | Data parallelism for concurrent size calculation |
| [chrono](https://crates.io/crates/chrono) | Date/time handling for last-modified timestamps |
| [serde](https://crates.io/crates/serde) + [serde_json](https://crates.io/crates/serde_json) | Serialization for config and JSON output |
| [dirs](https://crates.io/crates/dirs) | Cross-platform home/config directory resolution |
| [anyhow](https://crates.io/crates/anyhow) | Ergonomic error handling |

Terminal colors, table rendering, spinners, and input prompts are implemented without external crates using ANSI escape sequences and Unicode box-drawing characters.

## Testing

```bash
# Run all 137 tests
cargo test

# Run a specific test file
cargo test --test scanner_detection_test

# Run tests matching a pattern
cargo test format_bytes
```

Tests create temporary directories under the system temp dir (`/tmp/dev_sweep_test_*`) and clean up after themselves. No tests touch real project directories.

## Building for Release

```bash
cargo build --release
```

The release profile enables LTO, maximum optimization, and symbol stripping for a small, fast binary.

## License

MIT — see [LICENSE](LICENSE) for details.
