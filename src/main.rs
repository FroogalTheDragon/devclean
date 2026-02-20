use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{Parser, Subcommand};

use devclean::cleaner::clean_projects;
use devclean::config::DevCleanConfig;
use devclean::scanner::{ScannedProject, scan_directory};
use devclean::tui::display::{
    blue, confirm, cyan, dim, green, multi_select, print_clean_summary, print_results_table,
    red_bold, yellow_bold,
};
use devclean::util::{format_bytes, parse_age};

// ── CLI definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "devclean",
    about = "🧹 Find and clean build artifacts & dependency caches across all your dev projects",
    long_about = "devclean scans your filesystem for developer projects and identifies \
                  reclaimable disk space from build artifacts, dependency caches, and \
                  generated files. It supports 17+ project types including Rust, Node.js, \
                  Python, Java, .NET, Go, and more.",
    version,
    author = "Mark Waid Jr"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Directory to scan (defaults to current directory)
    #[arg(global = true)]
    path: Option<PathBuf>,

    /// Maximum directory depth to scan
    #[arg(short = 'd', long, global = true)]
    max_depth: Option<usize>,

    /// Only show projects older than this (e.g. "30d", "3m", "1y")
    #[arg(short, long, global = true)]
    older_than: Option<String>,

    /// Output results as JSON
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan for projects and show what can be cleaned (default)
    Scan,
    /// Interactively select and clean projects
    Clean {
        /// Clean all found projects without prompting
        #[arg(short, long)]
        all: bool,
        /// Show what would be cleaned without actually deleting
        #[arg(long)]
        dry_run: bool,
    },
    /// Show a quick summary of reclaimable space
    Summary,
    /// Manage devclean configuration
    Config {
        /// Show the current config
        #[arg(long)]
        show: bool,
        /// Reset config to defaults
        #[arg(long)]
        reset: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("  {} {}", red_bold("Error:"), e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let _config = DevCleanConfig::load();

    let scan_path = cli
        .path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let scan_path = if scan_path.starts_with("~") {
        dirs::home_dir()
            .unwrap_or_default()
            .join(scan_path.strip_prefix("~").unwrap_or(&scan_path))
    } else {
        scan_path
    };

    if !scan_path.is_dir() {
        anyhow::bail!(
            "Path does not exist or is not a directory: {}",
            scan_path.display()
        );
    }

    match cli.command.unwrap_or(Commands::Scan) {
        Commands::Scan => cmd_scan(
            &scan_path,
            cli.max_depth,
            cli.older_than.as_deref(),
            cli.json,
        ),
        Commands::Clean { all, dry_run } => cmd_clean(
            &scan_path,
            cli.max_depth,
            cli.older_than.as_deref(),
            all,
            dry_run,
            cli.json,
        ),
        Commands::Summary => cmd_summary(
            &scan_path,
            cli.max_depth,
            cli.older_than.as_deref(),
            cli.json,
        ),
        Commands::Config { show, reset } => cmd_config(show, reset),
    }
}

// ── Commands ────────────────────────────────────────────────────────────────

fn cmd_scan(
    path: &PathBuf,
    max_depth: Option<usize>,
    older_than: Option<&str>,
    json: bool,
) -> Result<()> {
    let mut projects = scan_directory(path, max_depth)?;
    filter_by_age(&mut projects, older_than)?;
    sort_by_size(&mut projects);

    if json {
        println!("{}", serde_json::to_string_pretty(&projects)?);
    } else {
        print_results_table(&projects);
    }

    Ok(())
}

fn cmd_clean(
    path: &PathBuf,
    max_depth: Option<usize>,
    older_than: Option<&str>,
    all: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    let mut projects = scan_directory(path, max_depth)?;
    filter_by_age(&mut projects, older_than)?;
    sort_by_size(&mut projects);

    if projects.is_empty() {
        println!(
            "\n  {} No projects with cleanable artifacts found.\n",
            blue("ℹ")
        );
        return Ok(());
    }

    print_results_table(&projects);

    let selected_projects: Vec<&ScannedProject> = if all {
        if !dry_run {
            let total: u64 = projects.iter().map(|p| p.total_cleanable_bytes).sum();
            let confirmed = confirm(&format!(
                "Clean ALL {} projects? This will free {} and cannot be undone!",
                projects.len(),
                format_bytes(total),
            ))?;

            if !confirmed {
                println!("  {} Aborted.\n", red_bold("✗"));
                return Ok(());
            }
        }
        projects.iter().collect()
    } else {
        let items: Vec<String> = projects
            .iter()
            .map(|p| {
                format!(
                    "{} ({}) — {} [{}]",
                    p.name,
                    p.kind,
                    format_bytes(p.total_cleanable_bytes),
                    p.clean_targets
                        .iter()
                        .map(|t| t.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect();

        let selections = multi_select("Select projects to clean:", &items)?;

        if selections.is_empty() {
            println!("  {} Nothing selected.\n", blue("ℹ"));
            return Ok(());
        }

        if !dry_run {
            let sel_total: u64 = selections
                .iter()
                .map(|&i| projects[i].total_cleanable_bytes)
                .sum();
            let confirmed = confirm(&format!(
                "Clean {} projects? This will free {}.",
                selections.len(),
                format_bytes(sel_total),
            ))?;
            if !confirmed {
                println!("  {} Aborted.\n", red_bold("✗"));
                return Ok(());
            }
        }

        selections.iter().map(|&i| &projects[i]).collect()
    };

    let action = if dry_run { "Would clean" } else { "Cleaning" };
    println!(
        "\n  {} {} {} projects...\n",
        dim("→"),
        action,
        cyan(&selected_projects.len().to_string()),
    );

    let results = clean_projects(&selected_projects, dry_run);

    if json {
        let summary = serde_json::json!({
            "dry_run": dry_run,
            "projects_cleaned": results.len(),
            "total_bytes_freed": results.iter().map(|r| r.bytes_freed).sum::<u64>(),
            "errors": results.iter().flat_map(|r| r.errors.clone()).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_clean_summary(&results, dry_run);
    }

    Ok(())
}

fn cmd_summary(
    path: &PathBuf,
    max_depth: Option<usize>,
    older_than: Option<&str>,
    json: bool,
) -> Result<()> {
    let mut projects = scan_directory(path, max_depth)?;
    filter_by_age(&mut projects, older_than)?;

    let total_bytes: u64 = projects.iter().map(|p| p.total_cleanable_bytes).sum();
    let total_projects = projects.len();

    let mut by_kind: std::collections::HashMap<String, (usize, u64)> =
        std::collections::HashMap::new();
    for p in &projects {
        let entry = by_kind.entry(p.kind.to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += p.total_cleanable_bytes;
    }

    if json {
        let summary = serde_json::json!({
            "total_projects": total_projects,
            "total_reclaimable_bytes": total_bytes,
            "total_reclaimable_human": format_bytes(total_bytes),
            "by_kind": by_kind.iter().map(|(k, (count, bytes))| {
                serde_json::json!({
                    "kind": k,
                    "projects": count,
                    "reclaimable_bytes": bytes,
                    "reclaimable_human": format_bytes(*bytes),
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("\n  {} devclean summary for {}\n", "📊", path.display());
        println!(
            "  Total projects:     {}",
            cyan(&total_projects.to_string())
        );
        println!(
            "  Reclaimable space:  {}",
            yellow_bold(&format_bytes(total_bytes))
        );
        println!();

        if !by_kind.is_empty() {
            println!("  {}", dim("By project type:"));

            let mut sorted: Vec<_> = by_kind.iter().collect();
            sorted.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

            for (kind, (count, bytes)) in sorted {
                println!(
                    "    {:>12}  {} projects, {}",
                    kind,
                    cyan(&count.to_string()),
                    yellow_bold(&format_bytes(*bytes)),
                );
            }
            println!();
        }
    }

    Ok(())
}

fn cmd_config(show: bool, reset: bool) -> Result<()> {
    if reset {
        let config = DevCleanConfig::default();
        config.save()?;
        println!("  {} Config reset to defaults.", green("✓"));
        println!(
            "  {} {}",
            dim("→"),
            DevCleanConfig::config_path().display()
        );
        return Ok(());
    }

    if show {
        let config = DevCleanConfig::load();
        println!("{}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    let config_path = DevCleanConfig::config_path();
    println!("\n  {} devclean configuration\n", "⚙");
    println!("  Config file: {}", config_path.display());
    println!(
        "  Exists:      {}",
        if config_path.exists() {
            green("yes")
        } else {
            dim("no (using defaults)")
        }
    );

    let config = DevCleanConfig::load();
    println!("\n{}", serde_json::to_string_pretty(&config)?);
    println!(
        "\n  {} Use {} or {} to manage.\n",
        dim("→"),
        green("--show"),
        green("--reset")
    );

    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn sort_by_size(projects: &mut Vec<ScannedProject>) {
    projects.sort_by(|a, b| b.total_cleanable_bytes.cmp(&a.total_cleanable_bytes));
}

fn filter_by_age(projects: &mut Vec<ScannedProject>, older_than: Option<&str>) -> Result<()> {
    if let Some(age_str) = older_than {
        let duration = parse_age(age_str)?;
        let cutoff = chrono::Local::now() - duration;
        projects.retain(|p| p.last_modified < cutoff);
    }
    Ok(())
}
