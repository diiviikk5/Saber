//! Small human-friendly formatters used across the UI.

/// Formats a play duration the way a person would say it:
/// `0m`, `45m`, `3h 12m`, `38h`.
pub fn playtime(secs: u64) -> String {
    let minutes = secs / 60;
    let hours = minutes / 60;
    let rest = minutes % 60;
    match (hours, rest) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, _) if h >= 10 => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}
