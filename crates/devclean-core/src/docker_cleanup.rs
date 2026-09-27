use crate::command::{run_ok, LONG, QUICK};
use crate::docker::{anonymous_volumes, is_running, reclaimable, stale_builders, stale_container_ids};
use crate::format::human;

const PRUNES: [&[&str]; 3] = [
    &["container", "prune", "-f", "--filter", "until=24h"],
    &["image", "prune", "-a", "-f", "--filter", "until=24h"],
    &["builder", "prune", "-f", "--filter", "until=24h"],
];

/// Stopped testcontainers only: `-a` alone would include running ones, and `rm -f` would kill them.
const STALE_CONTAINER_LISTING: &[&str] = &[
    "ps",
    "-a",
    "--filter",
    "label=org.testcontainers=true",
    "--filter",
    "status=exited",
    "--filter",
    "status=created",
    "--filter",
    "status=dead",
    "--format",
    "{{.ID}} {{.RunningFor}}",
];

/// Removes stale test containers and builders, then prunes. Returns failure messages; never touches running containers or named volumes.
pub fn clean(dry_run: bool, on_line: &mut dyn FnMut(&str)) -> Vec<String> {
    if !is_running() {
        on_line("docker is not running, skipped");
        return Vec::new();
    }
    let mut failures = Vec::new();
    remove_stale_containers(dry_run, on_line, &mut failures);
    remove_stale_builders(dry_run, on_line, &mut failures);
    if dry_run {
        on_line(&format!(
            "would prune containers, images and build cache older than 24h (up to {})",
            human(reclaimable())
        ));
    } else {
        for args in PRUNES {
            if let Err(err) = run_ok("docker", args, LONG) {
                failures.push(err.to_string());
            }
        }
    }
    remove_anonymous_volumes(dry_run, on_line, &mut failures);
    failures
}

/// Runs after the prunes so volumes freed by removed containers are unused by now.
fn remove_anonymous_volumes(dry_run: bool, on_line: &mut dyn FnMut(&str), failures: &mut Vec<String>) {
    let volumes = match run_ok("docker", &["volume", "ls", "-q", "--filter", "dangling=true"], QUICK) {
        Ok(listing) => anonymous_volumes(&listing),
        Err(err) => {
            failures.push(err.to_string());
            return;
        }
    };
    if volumes.is_empty() {
        return;
    }
    if dry_run {
        on_line(&format!("would remove {} unused anonymous volumes", volumes.len()));
        return;
    }
    let mut args = vec!["volume", "rm"];
    args.extend(volumes.iter().map(String::as_str));
    match run_ok("docker", &args, LONG) {
        Ok(_) => on_line(&format!("removed {} unused anonymous volumes", volumes.len())),
        Err(err) => failures.push(err.to_string()),
    }
}

fn remove_stale_containers(dry_run: bool, on_line: &mut dyn FnMut(&str), failures: &mut Vec<String>) {
    let listing = run_ok("docker", STALE_CONTAINER_LISTING, QUICK);
    let ids = match listing {
        Ok(output) => stale_container_ids(&output),
        Err(err) => {
            failures.push(err.to_string());
            return;
        }
    };
    if ids.is_empty() {
        return;
    }
    if dry_run {
        on_line(&format!("would remove {} stale test containers", ids.len()));
        return;
    }
    let mut args = vec!["rm", "-f"];
    args.extend(ids.iter().map(String::as_str));
    match run_ok("docker", &args, LONG) {
        Ok(_) => on_line(&format!("removed {} stale test containers", ids.len())),
        Err(err) => failures.push(err.to_string()),
    }
}

fn remove_stale_builders(dry_run: bool, on_line: &mut dyn FnMut(&str), failures: &mut Vec<String>) {
    let builders = run_ok("docker", &["buildx", "ls", "--format", "json"], QUICK).and_then(|out| stale_builders(&out));
    let builders = match builders {
        Ok(builders) => builders,
        Err(err) => {
            failures.push(err.to_string());
            return;
        }
    };
    for builder in builders {
        if dry_run {
            on_line(&format!("would remove builder {builder}"));
            continue;
        }
        match run_ok("docker", &["buildx", "rm", &builder], QUICK) {
            Ok(_) => on_line(&format!("removed builder {builder}")),
            Err(err) => failures.push(err.to_string()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_only_stopped_test_containers() {
        let filters: Vec<&str> = STALE_CONTAINER_LISTING
            .windows(2)
            .filter(|pair| pair[0] == "--filter")
            .map(|pair| pair[1])
            .collect();
        assert!(filters.contains(&"label=org.testcontainers=true"));
        assert!(filters.contains(&"status=exited"));
        assert!(!filters.iter().any(|filter| filter.contains("running")));
    }
}
