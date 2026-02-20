use std::io::{self, Write};

use crate::cleaner::CleanResult;
use crate::scanner::ScannedProject;

// ── ANSI color helpers ──────────────────────────────────────────────────────

fn bold(s: &str) -> String {
    format!("\x1b[1m{s}\x1b[0m")
}

fn green(s: &str) -> String {
    format!("\x1b[32m{s}\x1b[0m")
}

fn green_bold(s: &str) -> String {
    format!("\x1b[1;32m{s}\x1b[0m")
}

fn cyan(s: &str) -> String {
    format!("\x1b[36m{s}\x1b[0m")
}

fn cyan_bold(s: &str) -> String {
    format!("\x1b[1;36m{s}\x1b[0m")
}

fn yellow(s: &str) -> String {
    format!("\x1b[33m{s}\x1b[0m")
}

fn yellow_bold(s: &str) -> String {
    format!("\x1b[1;33m{s}\x1b[0m")
}

fn red(s: &str) -> String {
    format!("\x1b[31m{s}\x1b[0m")
}

fn dim(s: &str) -> String {
    format!("\x1b[2m{s}\x1b[0m")
}

fn blue(s: &str) -> String {
    format!("\x1b[34m{s}\x1b[0m")
}

fn _white_bold(s: &str) -> String {
    format!("\x1b[1;37m{s}\x1b[0m")
}

// ── Human-readable byte sizes ───────────────────────────────────────────────

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

// ── Table rendering ─────────────────────────────────────────────────────────

/// Visible length of a string (strips ANSI escape sequences).
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

/// Pad a string to a given visible width (right-padded).
fn pad_right(s: &str, width: usize) -> String {
    let vis = visible_len(s);
    if vis >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - vis))
    }
}

/// Pad a string to a given visible width (left-padded).
fn pad_left(s: &str, width: usize) -> String {
    let vis = visible_len(s);
    if vis >= width {
        s.to_string()
    } else {
        format!("{}{s}", " ".repeat(width - vis))
    }
}

struct TableRow {
    index: String,
    name: String,
    kind: String,
    size: String,
    targets: String,
    last_modified: String,
    path: String,
}

/// Print a formatted table of scanned projects.
pub fn print_results_table(projects: &[ScannedProject]) {
    if projects.is_empty() {
        println!(
            "\n  {} No projects with cleanable artifacts found.\n",
            blue("ℹ")
        );
        return;
    }

    let total_bytes: u64 = projects.iter().map(|p| p.total_cleanable_bytes).sum();
    let total_projects = projects.len();

    println!(
        "\n  {} Found {} projects with {} of reclaimable space\n",
        green_bold("✓"),
        cyan_bold(&total_projects.to_string()),
        yellow_bold(&format_bytes(total_bytes)),
    );

    let now = chrono::Local::now();

    let rows: Vec<TableRow> = projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let age = now.signed_duration_since(p.last_modified);
            let age_str = format_age(age);

            let targets_str = p
                .clean_targets
                .iter()
                .map(|t| format!("{} ({})", t.name, format_bytes(t.size_bytes)))
                .collect::<Vec<_>>()
                .join(", ");

            let display_path = shorten_path(&p.path.display().to_string());

            TableRow {
                index: format!("{}", i + 1),
                name: p.name.clone(),
                kind: p.kind.to_string(),
                size: format_bytes(p.total_cleanable_bytes),
                targets: targets_str,
                last_modified: age_str,
                path: display_path,
            }
        })
        .collect();

    // Calculate column widths
    let headers = ["#", "Project", "Type", "Cleanable", "Targets", "Last Modified", "Path"];
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

    for row in &rows {
        widths[0] = widths[0].max(row.index.len());
        widths[1] = widths[1].max(row.name.len());
        widths[2] = widths[2].max(row.kind.len());
        widths[3] = widths[3].max(row.size.len());
        widths[4] = widths[4].max(row.targets.len());
        widths[5] = widths[5].max(row.last_modified.len());
        widths[6] = widths[6].max(row.path.len());
    }

    // Clamp targets column to prevent insanely wide tables
    widths[4] = widths[4].min(50);
    widths[6] = widths[6].min(45);

    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Top border
    write!(out, "  ╭").unwrap();
    for (i, w) in widths.iter().enumerate() {
        write!(out, "{}", "─".repeat(w + 2)).unwrap();
        if i < widths.len() - 1 {
            write!(out, "┬").unwrap();
        }
    }
    writeln!(out, "╮").unwrap();

    // Header row
    write!(out, "  │").unwrap();
    for (i, header) in headers.iter().enumerate() {
        let padded = if i == 0 || i == 3 {
            pad_left(&bold(header), widths[i] + 8) // bold adds 8 chars of ANSI
        } else {
            pad_right(&bold(header), widths[i] + 8)
        };
        write!(out, " {padded} │").unwrap();
    }
    writeln!(out).unwrap();

    // Header separator
    write!(out, "  ├").unwrap();
    for (i, w) in widths.iter().enumerate() {
        write!(out, "{}", "─".repeat(w + 2)).unwrap();
        if i < widths.len() - 1 {
            write!(out, "┼").unwrap();
        }
    }
    writeln!(out, "┤").unwrap();

    // Data rows
    for row in &rows {
        let fields = [
            pad_left(&dim(&row.index), widths[0] + 8),
            pad_right(&row.name, widths[1]),
            pad_right(&cyan(&row.kind), widths[2] + 9), // cyan adds 9 chars
            pad_left(&yellow(&row.size), widths[3] + 9),
            pad_right(&truncate(&row.targets, widths[4]), widths[4]),
            pad_right(&dim(&row.last_modified), widths[5] + 8),
            pad_right(&dim(&truncate(&row.path, widths[6])), widths[6] + 8),
        ];

        write!(out, "  │").unwrap();
        for field in &fields {
            write!(out, " {field} │").unwrap();
        }
        writeln!(out).unwrap();
    }

    // Bottom border
    write!(out, "  ╰").unwrap();
    for (i, w) in widths.iter().enumerate() {
        write!(out, "{}", "─".repeat(w + 2)).unwrap();
        if i < widths.len() - 1 {
            write!(out, "┴").unwrap();
        }
    }
    writeln!(out, "╯").unwrap();

    writeln!(out).unwrap();
}

/// Print a summary after cleaning.
pub fn print_clean_summary(results: &[CleanResult], dry_run: bool) {
    let total_freed: u64 = results.iter().map(|r| r.bytes_freed).sum();
    let total_targets: usize = results.iter().map(|r| r.targets_cleaned).sum();
    let total_errors: usize = results.iter().map(|r| r.errors.len()).sum();

    if dry_run {
        println!(
            "\n  {} Dry run complete. {} would be freed from {} targets across {} projects.",
            bold("🔍"),
            yellow_bold(&format_bytes(total_freed)),
            cyan(&total_targets.to_string()),
            cyan(&results.len().to_string()),
        );
        println!(
            "  {} Run without {} to actually clean.\n",
            dim("→"),
            green("--dry-run"),
        );
    } else {
        println!(
            "\n  {} Cleaned! {} freed from {} targets across {} projects.",
            bold("🧹"),
            green_bold(&format_bytes(total_freed)),
            cyan(&total_targets.to_string()),
            cyan(&results.len().to_string()),
        );

        if total_errors > 0 {
            println!(
                "  {} {} errors occurred:",
                yellow("⚠"),
                total_errors,
            );
            for result in results {
                for error in &result.errors {
                    println!("    {} {}: {}", red("✗"), result.project_name, error);
                }
            }
        }
        println!();
    }
}

// ── Prompt helpers (no external crate) ──────────────────────────────────────

/// Display a multi-select prompt. Returns the indices selected.
pub fn multi_select(prompt: &str, items: &[String]) -> anyhow::Result<Vec<usize>> {
    println!("\n  {}", bold(prompt));
    println!("  {}\n", dim("Enter numbers separated by commas/spaces, ranges with dash (e.g. 1,3,5-8), or 'all'"));

    for (i, item) in items.iter().enumerate() {
        println!("    {}  {}", cyan_bold(&format!("{:>3}", i + 1)), item);
    }

    print!("\n  {} ", green_bold("❯"));
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Ok(Vec::new());
    }

    if input.eq_ignore_ascii_case("all") {
        return Ok((0..items.len()).collect());
    }

    let mut selected = Vec::new();
    for part in input.split([',', ' ']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, end)) = part.split_once('-') {
            let start: usize = start.trim().parse().map_err(|_| {
                anyhow::anyhow!("Invalid number: '{}'", start.trim())
            })?;
            let end: usize = end.trim().parse().map_err(|_| {
                anyhow::anyhow!("Invalid number: '{}'", end.trim())
            })?;
            if start < 1 || end > items.len() || start > end {
                anyhow::bail!("Invalid range: {}-{}", start, end);
            }
            for i in start..=end {
                selected.push(i - 1);
            }
        } else {
            let num: usize = part.parse().map_err(|_| {
                anyhow::anyhow!("Invalid number: '{}'", part)
            })?;
            if num < 1 || num > items.len() {
                anyhow::bail!("Number out of range: {}", num);
            }
            selected.push(num - 1);
        }
    }

    selected.sort();
    selected.dedup();
    Ok(selected)
}

/// Display a yes/no confirmation prompt.
pub fn confirm(prompt: &str) -> anyhow::Result<bool> {
    print!("  {} {} {} ", yellow("⚠"), prompt, dim("[y/N]"));
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes"))
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Format a duration into a human-readable age string.
fn format_age(duration: chrono::TimeDelta) -> String {
    let days = duration.num_days();
    if days > 365 {
        format!("{:.1}y ago", days as f64 / 365.0)
    } else if days > 30 {
        format!("{}mo ago", days / 30)
    } else if days > 0 {
        format!("{}d ago", days)
    } else {
        let hours = duration.num_hours();
        if hours > 0 {
            format!("{}h ago", hours)
        } else {
            "just now".to_string()
        }
    }
}

/// Shorten a path by replacing the home directory with ~.
fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        if path.starts_with(&home_str) {
            return path.replacen(&home_str, "~", 1);
        }
    }
    path.to_string()
}

/// Truncate a string to a max visible width, appending "…" if truncated.
fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 1 {
        format!("{}…", &s[..max_width - 1])
    } else {
        "…".to_string()
    }
}
