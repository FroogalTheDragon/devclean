# 🧹 devclean

**A fast, interactive CLI tool to find and clean build artifacts & dependency caches across all your dev projects.**

Ever wonder where all your disk space went? If you're a developer, chances are it's hiding in hundreds of `node_modules/`, `target/`, `.venv/`, and `build/` directories scattered across old projects you haven't touched in months.

`devclean` scans your filesystem, finds those space hogs, and lets you reclaim gigabytes of disk space with a single command.

## ✨ Features

- **🔍 Smart Detection** — Automatically detects 17+ project types (Rust, Node.js, Python, Java, .NET, Go, Zig, and more)
- **⚡ Fast** — Parallel filesystem scanning powered by `rayon`
- **📊 Beautiful Output** — Sorted tables showing reclaimable space per project
- **🎯 Interactive** — Multi-select which projects to clean, or clean everything at once
- **🛡️ Safe by Default** — Dry-run mode, confirmation prompts, never touches source code
- **📅 Age Filtering** — Only clean projects older than a specified age (e.g. `--older-than 3m`)
- **🔧 Configurable** — Persistent config for ignored paths, default scan roots, and more
- **📋 JSON Output** — Machine-readable output for scripting and automation

## 📦 Installation

### From source (requires Rust)

```bash
cargo install --path .
```

### Build locally

```bash
git clone https://github.com/markwaidjr/devclean.git
cd devclean
cargo build --release
# Binary will be at ./target/release/devclean
```

## 🚀 Usage

### Scan (default)

See what can be cleaned:

```bash
# Scan current directory
devclean

# Scan a specific directory
devclean ~/projects

# Scan with depth limit
devclean ~/projects -d 3

# Only show stale projects (older than 3 months)
devclean ~/projects --older-than 3m
```

### Clean

Interactively select and clean projects:

```bash
# Interactive mode — select which projects to clean
devclean clean

# Clean everything without prompting
devclean clean --all

# Dry run — see what would be cleaned without deleting
devclean clean --dry-run

# Clean projects older than 6 months
devclean clean --older-than 6m
```

### Summary

Quick overview of reclaimable space by project type:

```bash
devclean summary ~/projects
```

### Config

Manage persistent settings:

```bash
# Show config
devclean config --show

# Reset to defaults
devclean config --reset
```

## 🗂️ Supported Project Types

| Project Type | Marker Files | Cleaned Directories |
|---|---|---|
| **Rust** | `Cargo.toml` | `target/` |
| **Node.js** | `package.json` | `node_modules/`, `.next/`, `.nuxt/`, `dist/`, `.cache/` |
| **Python** | `pyproject.toml`, `setup.py`, `requirements.txt` | `__pycache__/`, `.venv/`, `venv/`, `.tox/`, `.mypy_cache/`, `.pytest_cache/` |
| **Java** | `pom.xml`, `build.gradle` | `target/`, `build/`, `.gradle/` |
| **.NET** | `*.csproj`, `*.fsproj`, `*.sln` | `bin/`, `obj/` |
| **Go** | `go.mod` | *(detected but no artifacts to clean)* |
| **Zig** | `build.zig` | `zig-cache/`, `zig-out/` |
| **CMake** | `CMakeLists.txt` | `build/`, `cmake-build-*/` |
| **Swift** | `Package.swift` | `.build/` |
| **Elixir** | `mix.exs` | `_build/`, `deps/` |
| **Haskell** | `stack.yaml`, `*.cabal` | `.stack-work/` |
| **Dart** | `pubspec.yaml` | `.dart_tool/`, `build/` |
| **Ruby** | `Gemfile` | `vendor/bundle/` |
| **Scala** | `build.sbt` | `target/`, `project/target/` |
| **Unity** | `ProjectSettings/ProjectVersion.txt` | `Library/`, `Temp/`, `Obj/`, `Logs/` |
| **Godot** | `project.godot` | `.godot/` |
| **Terraform** | `main.tf`, `*.tf` | `.terraform/` |

## 🔧 Configuration

devclean stores configuration at `~/.config/devclean/config.json`:

```json
{
  "ignore_paths": ["/path/to/skip"],
  "exclude_kinds": ["Go"],
  "default_roots": ["~/projects", "~/work"],
  "max_depth": null
}
```

## 📋 JSON Output

All commands support `--json` for machine-readable output, perfect for scripting:

```bash
devclean --json | jq '.[].total_cleanable_bytes' | paste -sd+ | bc
```

## 🏗️ Architecture

```
src/
├── main.rs           # CLI entry point (clap)
├── scanner/
│   ├── project.rs    # Project types, detection markers, clean targets
│   └── walk.rs       # Filesystem walker with parallel analysis
├── cleaner/
│   └── mod.rs        # Safe deletion with dry-run support
├── config/
│   └── mod.rs        # Persistent JSON configuration
└── tui/
    └── display.rs    # Table formatting and colored output
```

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions are welcome! Some ideas:

- [ ] Add more project types (PHP/Composer, Unreal Engine, etc.)
- [ ] Interactive TUI mode with `ratatui` (full-screen browsing)
- [ ] Cron/scheduled cleaning mode
- [ ] Protected path safelist
- [ ] Integration tests with temp directories
