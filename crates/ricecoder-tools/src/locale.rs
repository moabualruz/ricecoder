//! Locale formatting utilities matching OpenCode behavior
//!
//! This module provides centralized locale formatting utilities that match
//! OpenCode's `Locale` namespace for date/time, numbers, durations, and pluralization.

use chrono::{DateTime, Local, Timelike, Datelike};

/// Locale formatting utilities matching OpenCode behavior
pub struct Locale;

impl Locale {
    /// Format titlecase (capitalize first letter of each word)
    pub fn titlecase(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().chain(chars).collect::<String>()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Format time using local timezone (short format)
    /// Matches: `new Date(input).toLocaleTimeString(undefined, { timeStyle: "short" })`
    pub fn time(timestamp_ms: i64) -> String {
        let datetime = Self::timestamp_to_local(timestamp_ms);
        format!("{:02}:{:02}", datetime.hour(), datetime.minute())
    }

    /// Format datetime with time and date
    /// Matches: `${time(input)} · ${date.toLocaleDateString()}`
    pub fn datetime(timestamp_ms: i64) -> String {
        let datetime = Self::timestamp_to_local(timestamp_ms);
        let time_str = Self::time(timestamp_ms);
        let date_str = format!(
            "{}/{}/{}",
            datetime.month(),
            datetime.day(),
            datetime.year()
        );
        format!("{} · {}", time_str, date_str)
    }

    /// Format as time if today, otherwise datetime
    /// Matches OpenCode's comparison logic
    pub fn today_time_or_datetime(timestamp_ms: i64) -> String {
        let datetime = Self::timestamp_to_local(timestamp_ms);
        let now = Local::now();

        let is_today = datetime.year() == now.year()
            && datetime.month() == now.month()
            && datetime.day() == now.day();

        if is_today {
            Self::time(timestamp_ms)
        } else {
            Self::datetime(timestamp_ms)
        }
    }

    /// Format number with K/M abbreviations
    /// Matches: `(num/1_000_000).toFixed(1) + "M"` or `(num/1_000).toFixed(1) + "K"`
    pub fn number(num: f64) -> String {
        if num >= 1_000_000.0 {
            format!("{:.1}M", num / 1_000_000.0)
        } else if num >= 1_000.0 {
            format!("{:.1}K", num / 1_000.0)
        } else {
            num.to_string()
        }
    }

    /// Format duration in milliseconds to human-readable format
    /// Matches OpenCode's duration logic (with known inconsistency in days calculation)
    pub fn duration(ms: u64) -> String {
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60_000 {
            format!("{:.1}s", ms as f64 / 1000.0)
        } else if ms < 3_600_000 {
            let minutes = ms / 60_000;
            let seconds = (ms % 60_000) / 1000;
            format!("{}m {}s", minutes, seconds)
        } else if ms < 86_400_000 {
            let hours = ms / 3_600_000;
            let minutes = (ms % 3_600_000) / 60_000;
            format!("{}h {}m", hours, minutes)
        } else {
            // Note: Replicating OpenCode's behavior including the apparent inconsistency
            // in day/hour calculation (days derived from unexpected modulo base)
            let hours = ms / 3_600_000;
            let days = (ms % 3_600_000) / 86_400_000;
            format!("{}d {}h", days, hours)
        }
    }

    /// Truncate string to length with ellipsis
    pub fn truncate(s: &str, len: usize) -> String {
        if s.len() <= len {
            s.to_string()
        } else {
            format!("{}…", &s[..len.saturating_sub(1)])
        }
    }

    /// Truncate string in the middle with ellipsis
    /// Matches: `str.slice(0, keepStart) + ellipsis + str.slice(-keepEnd)`
    pub fn truncate_middle(s: &str, max_length: usize) -> String {
        let char_count = s.chars().count();
        if char_count <= max_length {
            return s.to_string();
        }

        let ellipsis = "…";
        let ellipsis_len = 1; // ellipsis is 1 character
        let keep_start = (max_length.saturating_sub(ellipsis_len) + 1) / 2;
        let keep_end = (max_length.saturating_sub(ellipsis_len)) / 2;

        let chars: Vec<char> = s.chars().collect();
        let start: String = chars.iter().take(keep_start).collect();
        let end: String = chars.iter().rev().take(keep_end).rev().collect();

        format!("{}{}{}", start, ellipsis, end)
    }

    /// Pluralize with count replacement
    /// Matches: `count === 1 ? singular : plural` with `"{}"` replacement
    pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
        let template = if count == 1 { singular } else { plural };
        template.replace("{}", &count.to_string())
    }

    // Helper to convert timestamp to local DateTime
    fn timestamp_to_local(timestamp_ms: i64) -> DateTime<Local> {
        let secs = timestamp_ms / 1000;
        let nanos = ((timestamp_ms % 1000) * 1_000_000) as u32;
        DateTime::from_timestamp(secs, nanos)
            .expect("Invalid timestamp")
            .with_timezone(&Local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titlecase() {
        assert_eq!(Locale::titlecase("hello world"), "Hello World");
        assert_eq!(Locale::titlecase("the quick brown fox"), "The Quick Brown Fox");
        assert_eq!(Locale::titlecase(""), "");
        assert_eq!(Locale::titlecase("a"), "A");
    }

    #[test]
    fn test_time_formatting() {
        // Test with a known timestamp: 2024-01-15 14:30:00 UTC
        let timestamp_ms = 1705331400000;
        let time_str = Locale::time(timestamp_ms);
        // Time format should be HH:MM (locale-dependent, but should contain colon)
        assert!(time_str.contains(':'));
    }

    #[test]
    fn test_datetime_formatting() {
        let timestamp_ms = 1705331400000;
        let datetime_str = Locale::datetime(timestamp_ms);
        // Should contain both time and date parts with separator
        assert!(datetime_str.contains('·'));
        assert!(datetime_str.contains('/') || datetime_str.contains('-'));
    }

    #[test]
    fn test_number_formatting() {
        assert_eq!(Locale::number(500.0), "500");
        assert_eq!(Locale::number(1_500.0), "1.5K");
        assert_eq!(Locale::number(1_000_000.0), "1.0M");
        assert_eq!(Locale::number(2_500_000.0), "2.5M");
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(Locale::duration(500), "500ms");
        assert_eq!(Locale::duration(1_500), "1.5s");
        assert_eq!(Locale::duration(90_000), "1m 30s");
        assert_eq!(Locale::duration(3_660_000), "1h 1m");
        // Note: day calculation matches OpenCode's behavior
        assert!(Locale::duration(90_000_000).contains('d'));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(Locale::truncate("hello", 10), "hello");
        assert_eq!(Locale::truncate("hello world", 8), "hello w…");
        assert_eq!(Locale::truncate("test", 3), "te…");
    }

    #[test]
    fn test_truncate_middle() {
        assert_eq!(Locale::truncate_middle("hello", 10), "hello");
        assert_eq!(Locale::truncate_middle("very long string here", 10), "very …here");
        assert_eq!(Locale::truncate_middle("abcdefghijk", 7), "abc…ijk");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(Locale::pluralize(0, "{} item", "{} items"), "0 items");
        assert_eq!(Locale::pluralize(1, "{} item", "{} items"), "1 item");
        assert_eq!(Locale::pluralize(5, "{} item", "{} items"), "5 items");
    }
}
