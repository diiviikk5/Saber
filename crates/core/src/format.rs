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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playtime_reads_naturally() {
        assert_eq!(playtime(0), "0m");
        assert_eq!(playtime(59), "0m");
        assert_eq!(playtime(45 * 60), "45m");
        assert_eq!(playtime(3 * 3600), "3h");
        assert_eq!(playtime(3 * 3600 + 12 * 60), "3h 12m");
        assert_eq!(playtime(38 * 3600 + 5 * 60), "38h");
    }

    #[test]
    fn relative_buckets() {
        let now = 1_000_000_000;
        assert_eq!(relative(now, now), "just now");
        assert_eq!(relative(now - 60, now), "1 minute ago");
        assert_eq!(relative(now - 12 * 60, now), "12 minutes ago");
        assert_eq!(relative(now - 3 * 3600, now), "3 hours ago");
        assert_eq!(relative(now - 30 * 3600, now), "yesterday");
        assert_eq!(relative(now - 3 * 86400, now), "3 days ago");
        assert_eq!(relative(now - 15 * 86400, now), "2 weeks ago");
        assert_eq!(relative(now - 400 * 86400, now), "1 year ago");
    }

    #[test]
    fn relative_handles_future_timestamps() {
        assert_eq!(relative(10, 5), "just now");
    }
}
