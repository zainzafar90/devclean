use std::collections::HashMap;

use serde::Serialize;

use crate::command::{run_ok, QUICK};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProcessUsage {
    pub name: String,
    pub bytes: u64,
}

/// Resident memory summed per executable name, largest first.
pub fn top_processes(limit: usize) -> Result<Vec<ProcessUsage>> {
    let output = run_ok("ps", &["-Ao", "rss=,comm="], QUICK)?;
    let mut usage = parse_ps(&output);
    usage.truncate(limit);
    Ok(usage)
}

pub fn parse_ps(output: &str) -> Vec<ProcessUsage> {
    let mut totals: HashMap<&str, u64> = HashMap::new();
    for line in output.lines() {
        let line = line.trim_start();
        let Some((rss, command)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(kb) = rss.parse::<u64>() else {
            continue;
        };
        let name = command.trim().rsplit('/').next().unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        *totals.entry(name).or_default() += kb * 1024;
    }
    let mut usage: Vec<ProcessUsage> = totals
        .into_iter()
        .map(|(name, bytes)| ProcessUsage {
            name: name.to_string(),
            bytes,
        })
        .collect();
    usage.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.name.cmp(&b.name)));
    usage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_by_executable_name() {
        let output = "  100 /Applications/Foo.app/Contents/MacOS/Foo Helper\n\
                      300 /usr/bin/bar\n\
                      50 /Applications/Foo.app/Contents/MacOS/Foo Helper\n\
                      garbage\n";
        let usage = parse_ps(output);
        assert_eq!(
            usage[0],
            ProcessUsage {
                name: "bar".into(),
                bytes: 300 * 1024
            }
        );
        assert_eq!(
            usage[1],
            ProcessUsage {
                name: "Foo Helper".into(),
                bytes: 150 * 1024
            }
        );
        assert_eq!(usage.len(), 2);
    }

    #[test]
    fn lists_live_processes() {
        assert!(!top_processes(3).unwrap().is_empty());
    }
}
