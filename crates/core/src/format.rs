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

/// Describes how long ago `then` was relative to `now` (both unix seconds):
/// `just now`, `12 minutes ago`, `yesterday`, `3 weeks ago`.
pub fn relative(then: u64, now: u64) -> String {
    let delta = now.saturating_sub(then);
    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;
    const MONTH: u64 = 30 * DAY;
    const YEAR: u64 = 365 * DAY;

    fn plural(n: u64, unit: &str) -> String {
        if n == 1 {
            format!("1 {unit} ago")
        } else {
            format!("{n} {unit}s ago")
        }
    }

    match delta {
        d if d < MIN => "just now".into(),
        d if d < HOUR => plural(d / MIN, "minute"),
        d if d < DAY => plural(d / HOUR, "hour"),
        d if d < 2 * DAY => "yesterday".into(),
        d if d < WEEK => plural(d / DAY, "day"),
        d if d < MONTH => plural(d / WEEK, "week"),
        d if d < YEAR => plural(d / MONTH, "month"),
        d => plural(d / YEAR, "year"),
    }
}

/// Current unix time in seconds.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
