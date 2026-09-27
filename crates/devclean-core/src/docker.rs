use serde_json::Value;

use crate::command::{has, run, run_ok, QUICK};
use crate::error::{CoreError, Result};

pub const STALE_HOURS: u64 = 24;

pub fn is_running() -> bool {
    has("docker") && run("docker", &["info"], QUICK).is_ok_and(|out| out.status.success())
}

/// Bytes `docker system df` says could be reclaimed; 0 when Docker is not running.
pub fn reclaimable() -> u64 {
    if !is_running() {
        return 0;
    }
    run_ok("docker", &["system", "df", "--format", "{{.Reclaimable}}"], QUICK)
        .map(|out| parse_reclaimable(&out))
        .unwrap_or(0)
}

/// Sums lines like `1.2GB (50%)`; Docker uses decimal units.
pub fn parse_reclaimable(output: &str) -> u64 {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter_map(parse_decimal_size)
        .sum()
}

pub fn parse_decimal_size(token: &str) -> Option<u64> {
    let split = token.find(|c: char| !(c.is_ascii_digit() || c == '.'))?;
    let (number, unit) = token.split_at(split);
    let value: f64 = number.parse().ok()?;
    let multiplier = match unit {
        "B" => 1.0,
        "kB" | "KB" => 1e3,
        "MB" => 1e6,
        "GB" => 1e9,
        "TB" => 1e12,
        _ => return None,
    };
    Some((value * multiplier) as u64)
}

/// Hours in Docker's `RunningFor` text (`25 hours ago`, `3 days ago`, `About an hour ago`).
pub fn running_for_hours(text: &str) -> Option<u64> {
    let text = text.trim().trim_end_matches(" ago");
    if text.starts_with("Less than") || text.contains("second") || text.contains("minute") {
        return Some(0);
    }
    if text == "About an hour" {
        return Some(1);
    }
    let (count, unit) = text.split_once(' ')?;
    let count: u64 = count.parse().ok()?;
    let hours = match unit.trim_end_matches('s') {
        "hour" => 1,
        "day" => 24,
        "week" => 24 * 7,
        "month" => 24 * 30,
        "year" => 24 * 365,
        _ => return None,
    };
    Some(count * hours)
}

/// Container ids from `docker ps --format '{{.ID}} {{.RunningFor}}'` that are a day old or more.
pub fn stale_container_ids(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_once(' '))
        .filter(|(_, age)| running_for_hours(age).is_some_and(|hours| hours >= STALE_HOURS))
        .map(|(id, _)| id.to_string())
        .collect()
}

/// Names of `docker-container` builders whose first node is not running, from `docker buildx ls --format json`.
pub fn stale_builders(output: &str) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let builder: Value = serde_json::from_str(line).map_err(|err| CoreError::Parse {
            what: "docker buildx ls",
            detail: err.to_string(),
        })?;
        let driver = builder["Driver"].as_str().unwrap_or_default();
        let status = builder["Nodes"][0]["Status"].as_str().unwrap_or_default();
        if driver == "docker-container" && status != "running" {
            if let Some(name) = builder["Name"].as_str() {
                names.push(name.to_string());
            }
        }
    }
    Ok(names)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContainerMemory {
    pub name: String,
    pub usage: String,
}

pub fn container_memory() -> Result<Vec<ContainerMemory>> {
    let output = run_ok(
        "docker",
        &["stats", "--no-stream", "--format", "{{.MemUsage}}\t{{.Name}}"],
        QUICK,
    )?;
    Ok(output
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(usage, name)| ContainerMemory {
            name: name.to_string(),
            usage: usage.split(" /").next().unwrap_or(usage).to_string(),
        })
        .collect())
}

/// Anonymous volumes are named by a 64-character hex id; a named volume never is.
pub fn anonymous_volumes(listing: &str) -> Vec<String> {
    listing
        .lines()
        .map(str::trim)
        .filter(|name| name.len() == 64 && name.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(str::to_string)
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_running_for() {
        assert_eq!(running_for_hours("Less than a second ago"), Some(0));
        assert_eq!(running_for_hours("45 minutes ago"), Some(0));
        assert_eq!(running_for_hours("About an hour ago"), Some(1));
        assert_eq!(running_for_hours("23 hours ago"), Some(23));
        assert_eq!(running_for_hours("25 hours ago"), Some(25));
        assert_eq!(running_for_hours("2 days ago"), Some(48));
        assert_eq!(running_for_hours("1 week ago"), Some(168));
        assert_eq!(running_for_hours("3 months ago"), Some(2160));
        assert_eq!(running_for_hours("2 years ago"), Some(17520));
        assert_eq!(running_for_hours("soon"), None);
    }

    #[test]
    fn picks_containers_a_day_old() {
        let output = "aaa 23 hours ago\nbbb 24 hours ago\nccc 3 days ago\nddd About an hour ago\n";
        assert_eq!(stale_container_ids(output), vec!["bbb", "ccc"]);
    }

    #[test]
    fn keeps_named_volumes() {
        let anonymous = "f".repeat(64);
        let listing = format!("postgres-data\n{anonymous}\nredis\n{}\n", "a1".repeat(31));
        assert_eq!(anonymous_volumes(&listing), vec![anonymous]);
    }

    #[test]
    fn parses_reclaimable_sizes() {
        assert_eq!(parse_decimal_size("1.5GB"), Some(1_500_000_000));
        assert_eq!(parse_decimal_size("12kB"), Some(12_000));
        assert_eq!(parse_decimal_size("0B"), Some(0));
        assert_eq!(parse_decimal_size("n/a"), None);
        assert_eq!(parse_reclaimable("1GB (50%)\n500MB (100%)\n0B (0%)\n"), 1_500_000_000);
    }

    #[test]
    fn finds_stopped_container_builders() {
        let output = r#"{"Name":"default","Driver":"docker","Nodes":[{"Name":"default","Status":"running"}]}
{"Name":"ci","Driver":"docker-container","Nodes":[{"Name":"ci0","Status":"inactive"}]}
{"Name":"live","Driver":"docker-container","Nodes":[{"Name":"live0","Status":"running"}]}
{"Name":"empty","Driver":"docker-container","Nodes":[]}
"#;
        assert_eq!(stale_builders(output).unwrap(), vec!["ci", "empty"]);
        assert!(stale_builders("not json").is_err());
    }
}
