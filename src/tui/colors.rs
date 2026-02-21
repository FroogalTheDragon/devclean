// ── ANSI color helpers ──────────────────────────────────────────────────────

pub fn bold(s: &str) -> String {
    format!("\x1b[1m{s}\x1b[0m")
}

pub fn green(s: &str) -> String {
    format!("\x1b[32m{s}\x1b[0m")
}

pub fn green_bold(s: &str) -> String {
    format!("\x1b[1;32m{s}\x1b[0m")
}

pub fn cyan(s: &str) -> String {
    format!("\x1b[36m{s}\x1b[0m")
}

pub fn cyan_bold(s: &str) -> String {
    format!("\x1b[1;36m{s}\x1b[0m")
}

pub fn yellow(s: &str) -> String {
    format!("\x1b[33m{s}\x1b[0m")
}

pub fn yellow_bold(s: &str) -> String {
    format!("\x1b[1;33m{s}\x1b[0m")
}

pub fn red(s: &str) -> String {
    format!("\x1b[31m{s}\x1b[0m")
}

pub fn red_bold(s: &str) -> String {
    format!("\x1b[1;31m{s}\x1b[0m")
}

pub fn dim(s: &str) -> String {
    format!("\x1b[2m{s}\x1b[0m")
}

pub fn blue(s: &str) -> String {
    format!("\x1b[34m{s}\x1b[0m")
}