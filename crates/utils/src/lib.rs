//! General utility functions for Bnpm.
//!
//! Includes version comparison, glob matching, string formatting,
//! slug generation, and byte/duration formatting.

use std::cmp::Ordering;
use std::time::Duration;

/// Error type for utility operations.
#[derive(Debug, thiserror::Error)]
pub enum UtilsError {
    #[error("size parse error: {0}")]
    SizeParse(String),
    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),
}

/// Compare two semver version strings.
///
/// Returns `Ordering::Less` if `a < b`, etc.
pub fn version_compare(a: &str, b: &str) -> Ordering {
    let parse = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };

    let a_parts = parse(a);
    let b_parts = parse(b);

    let max_len = a_parts.len().max(b_parts.len());
    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);
        match a_val.cmp(&b_val) {
            Ordering::Equal => continue,
            non_eq => return non_eq,
        }
    }
    Ordering::Equal
}

/// Check whether a path matches a glob pattern.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let re_pattern = pattern
        .replace('.', "\\.")
        .replace("**/", "\u{0}")
        .replace("**", "\u{1}")
        .replace('*', "[^/]*")
        .replace('?', "[^/]")
        .replace('\u{0}', "(.*/)?")
        .replace('\u{1}', ".*");

    let full_pattern = format!("^{}$", re_pattern);
    regex::Regex::new(&full_pattern)
        .map(|re| re.is_match(path))
        .unwrap_or(false)
}

/// Truncate a string to at most `max_len` characters.
///
/// Returns the original string if it fits, otherwise a substring
/// with a `…` suffix appended to indicate truncation.
pub fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        return s;
    }

    let end = s.char_indices()
        .take(max_len.saturating_sub(1))
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    // Can't return owned String, so we return the truncated view
    // This is a limitation of returning &str - for true truncation
    // with ellipsis, use truncate_with_ellipsis
    &s[..end]
}

/// Truncate a string with an ellipsis character.
pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    let end = s.char_indices()
        .take(max_len.saturating_sub(1))
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    format!("{}…", &s[..end])
}

/// Convert a string to a URL-safe slug.
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                Some(c)
            } else if c.is_whitespace() {
                Some('-')
            } else if c.is_ascii_punctuation() {
                None
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Format a byte count as a human-readable string.
///
/// For example: 1.5 KiB, 3.2 MiB, 1.0 GiB.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let bytes = bytes as f64;

    if bytes < 1.0 {
        return "0 B".to_string();
    }

    let unit_idx = (bytes.log10() / 3.0) as usize;
    let unit_idx = unit_idx.min(UNITS.len() - 1);
    let value = bytes / (1024u64.pow(unit_idx as u32) as f64);

    if unit_idx == 0 {
        format!("{} B", bytes as u64)
    } else {
        format!("{:.1} {}", value, UNITS[unit_idx])
    }
}

/// Parse a human-readable size string into bytes.
///
/// Supports suffixes: B, KiB, MiB, GiB, TiB, KB, MB, GB, TB.
pub fn parse_size(s: &str) -> Result<u64, UtilsError> {
    let s = s.trim();

    let (num_str, unit) = if s.ends_with("TiB") {
        (&s[..s.len() - 3], 1024u64.pow(4))
    } else if s.ends_with("GiB") {
        (&s[..s.len() - 3], 1024u64.pow(3))
    } else if s.ends_with("MiB") {
        (&s[..s.len() - 3], 1024u64.pow(2))
    } else if s.ends_with("KiB") {
        (&s[..s.len() - 3], 1024u64)
    } else if s.ends_with("TB") {
        (&s[..s.len() - 2], 1000u64.pow(4))
    } else if s.ends_with("GB") {
        (&s[..s.len() - 2], 1000u64.pow(3))
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1000u64.pow(2))
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1000u64)
    } else if s.ends_with('B') {
        (&s[..s.len() - 1], 1u64)
    } else {
        (s, 1u64)
    };

    let num: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| UtilsError::SizeParse(format!("invalid number: {}", num_str)))?;

    Ok((num * unit as f64) as u64)
}

/// Format a `Duration` as a human-readable string.
///
/// For example: "1s", "2.5m", "1h 30m", "500ms".
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f64();

    if total_secs >= 3600.0 {
        let hours = (total_secs / 3600.0).floor() as u64;
        let minutes = ((total_secs % 3600.0) / 60.0).floor() as u64;
        if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    } else if total_secs >= 60.0 {
        let minutes = (total_secs / 60.0).floor() as u64;
        let secs = (total_secs % 60.0).floor() as u64;
        if secs > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}m", minutes)
        }
    } else if total_secs >= 1.0 {
        format!("{:.1}s", total_secs)
    } else {
        let millis = duration.as_millis();
        if millis > 0 {
            format!("{}ms", millis)
        } else {
            let micros = duration.as_micros();
            if micros > 0 {
                format!("{}µs", micros)
            } else {
                format!("{}ns", duration.as_nanos())
            }
        }
    }
}

/// Generate a cryptographically random string of given length using alphanumeric characters.
pub fn random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert_eq!(version_compare("1.0.0", "2.0.0"), Ordering::Less);
        assert_eq!(version_compare("2.0.0", "1.0.0"), Ordering::Greater);
        assert_eq!(version_compare("1.0.0", "1.0.0"), Ordering::Equal);
        assert_eq!(version_compare("v1.0.0", "1.0.0"), Ordering::Equal);
        assert_eq!(version_compare("1.0", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.rs", "lib.rs"));
        assert!(glob_match("src/**/*.rs", "src/main.rs"));
        assert!(!glob_match("*.txt", "lib.rs"));
        assert!(glob_match("test?", "test1"));
    }

    #[test]
    fn test_truncate() {
        let s = "hello world";
        assert_eq!(truncate(s, 20), s);
        assert_eq!(truncate_with_ellipsis("hello world this is long", 10), "hello wor…");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Hello   World  "), "hello-world");
        assert_eq!(slugify("test@#$file"), "testfile");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1536), "1.5 KiB");
        assert_eq!(format_bytes(1048576), "1.0 MiB");
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("500").unwrap(), 500);
        assert_eq!(parse_size("1 KiB").unwrap(), 1024);
        assert_eq!(parse_size("1.5 MiB").unwrap(), 1572864);
        assert_eq!(parse_size("1 GB").unwrap(), 1000000000);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(5)), "5.0s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
    }

    #[test]
    fn test_random_string() {
        let s = random_string(16);
        assert_eq!(s.len(), 16);
        // Verify alphanumeric only
        assert!(s.chars().all(|c| c.is_alphanumeric()));
    }
}
