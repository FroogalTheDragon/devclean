use anyhow::Result;

/// Parse an age string like "30d", "3m", "1y" into a chrono TimeDelta.
///
/// Supported units:
/// - `d` — days
/// - `w` — weeks
/// - `m` — months (30 days)
/// - `y` — years (365 days)
pub fn parse_age(s: &str) -> Result<chrono::TimeDelta> {
    let s = s.trim().to_lowercase();
    let (num_str, unit) = if s.ends_with('d') {
        (&s[..s.len() - 1], 'd')
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], 'm')
    } else if s.ends_with('y') {
        (&s[..s.len() - 1], 'y')
    } else if s.ends_with('w') {
        (&s[..s.len() - 1], 'w')
    } else {
        anyhow::bail!(
            "Invalid age format '{}'. Use e.g. '30d' (days), '4w' (weeks), '3m' (months), '1y' (years)",
            s
        );
    };

    let num: i64 = num_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid number in age string: '{}'", num_str))?;

    let days = match unit {
        'd' => num,
        'w' => num * 7,
        'm' => num * 30,
        'y' => num * 365,
        _ => unreachable!(),
    };

    chrono::TimeDelta::try_days(days).ok_or_else(|| anyhow::anyhow!("Duration too large"))
}

/// Format a byte count into a human-readable string (e.g. "1.5 GB").
pub fn format_bytes(bytes: u64) -> String {
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

/// Visible length of a string (strips ANSI escape sequences).
pub fn visible_len(s: &str) -> usize {
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
pub fn pad_right(s: &str, width: usize) -> String {
    let vis = visible_len(s);
    if vis >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - vis))
    }
}

/// Pad a string to a given visible width (left-padded).
pub fn pad_left(s: &str, width: usize) -> String {
    let vis = visible_len(s);
    if vis >= width {
        s.to_string()
    } else {
        format!("{}{s}", " ".repeat(width - vis))
    }
}

/// Format a duration into a human-readable age string.
pub fn format_age(duration: chrono::TimeDelta) -> String {
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

/// Truncate a string to a max visible width, appending "…" if truncated.
pub fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 1 {
        format!("{}…", &s[..max_width - 1])
    } else {
        "…".to_string()
    }
}

/// Shorten a path by replacing the home directory with ~.
pub fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        let home_str = home.display().to_string();
        if path.starts_with(&home_str) {
            return path.replacen(&home_str, "~", 1);
        }
    }
    path.to_string()
}
