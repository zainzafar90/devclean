const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

/// Binary units; GB and TB get one decimal, smaller units are truncated to whole numbers.
pub fn human(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit >= 3 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{} {}", value as u64, UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::human;

    #[test]
    fn formats_like_the_bash_version() {
        assert_eq!(human(0), "0 B");
        assert_eq!(human(1023), "1023 B");
        assert_eq!(human(1024), "1 KB");
        assert_eq!(human(1536), "1 KB");
        assert_eq!(human(409 * 1024 * 1024 + 900_000), "409 MB");
        assert_eq!(human(3 * 1024 * 1024 * 1024 / 2), "1.5 GB");
        assert_eq!(human(2 * 1024u64.pow(4)), "2.0 TB");
        assert_eq!(human(5000 * 1024u64.pow(4)), "5000.0 TB");
    }
}
