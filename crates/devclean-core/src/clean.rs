use serde::Serialize;

use crate::areas::{area_size, Area};
use crate::command::{has, run_ok, LONG};
use crate::context::Context;
use crate::path_guard::wipe_contents;
use crate::{docker_cleanup, worktree_cleanup};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanOutcome {
    pub area: Area,
    pub before: u64,
    pub after: u64,
    pub dry_run: bool,
    pub failures: Vec<String>,
}

impl CleanOutcome {
    pub fn freed(&self) -> u64 {
        self.before.saturating_sub(self.after)
    }
}

fn tool_command(area: Area) -> Option<&'static [&'static str]> {
    let command: &'static [&'static str] = match area {
        Area::Pnpm => &["pnpm", "store", "prune"],
        Area::Uv => &["uv", "cache", "prune"],
        Area::Npm => &["npm", "cache", "clean", "--force"],
        Area::Yarn => &["yarn", "cache", "clean"],
        Area::Brew => &["brew", "cleanup", "-s", "--prune=all"],
        Area::Pip => &["pip3", "cache", "purge"],
        Area::Simulators => &["xcrun", "simctl", "delete", "unavailable"],
        _ => return None,
    };
    Some(command)
}

fn wipes_paths(area: Area) -> bool {
    matches!(
        area,
        Area::Xcode | Area::Simulators | Area::Playwright | Area::Cocoapods
    )
}

/// Cleans one area. With `dry_run` nothing changes and each line says what would happen.
pub fn clean_area(ctx: &Context, area: Area, dry_run: bool, on_line: &mut dyn FnMut(&str)) -> CleanOutcome {
    let before = area_size(ctx, area);
    let mut failures = match area {
        Area::Docker => docker_cleanup::clean(dry_run, on_line),
        Area::Worktrees => worktree_cleanup::clean(ctx, dry_run, on_line),
        _ => Vec::new(),
    };
    if let Some(command) = tool_command(area) {
        failures.extend(run_tool(command, dry_run, on_line));
    }
    if wipes_paths(area) {
        failures.extend(wipe_area(ctx, area, dry_run, on_line));
    }
    let after = if dry_run { before } else { area_size(ctx, area) };
    CleanOutcome {
        area,
        before,
        after,
        dry_run,
        failures,
    }
}

fn run_tool(command: &[&str], dry_run: bool, on_line: &mut dyn FnMut(&str)) -> Option<String> {
    let (program, args) = command.split_first()?;
    if !has(program) {
        on_line(&format!("{program} not installed, skipped"));
        return None;
    }
    if dry_run {
        on_line(&format!("would run `{}`", command.join(" ")));
        return None;
    }
    run_ok(program, args, LONG).err().map(|err| err.to_string())
}

fn wipe_area(ctx: &Context, area: Area, dry_run: bool, on_line: &mut dyn FnMut(&str)) -> Vec<String> {
    let mut failures = Vec::new();
    for path in area.paths(&ctx.home).into_iter().filter(|path| path.is_dir()) {
        if dry_run {
            on_line(&format!("would empty {}", ctx.tilde(&path)));
            continue;
        }
        match wipe_contents(&path, &ctx.home) {
            Ok(errors) => failures.extend(errors),
            Err(err) => failures.push(err.to_string()),
        }
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_ctx(home: &std::path::Path) -> Context {
        Context {
            home: home.to_path_buf(),
            config_file: home.join("none"),
        }
    }

    #[test]
    fn dry_run_changes_nothing() {
        let home = tempfile::tempdir().unwrap();
        let cache = home.path().join("Library/Caches/CocoaPods");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("pod"), vec![0u8; 64 * 1024]).unwrap();
        let mut lines = Vec::new();
        let outcome = clean_area(&temp_ctx(home.path()), Area::Cocoapods, true, &mut |l| {
            lines.push(l.to_string())
        });
        assert_eq!(outcome.freed(), 0);
        assert!(cache.join("pod").exists());
        assert_eq!(lines, vec!["would empty ~/Library/Caches/CocoaPods"]);
    }

    #[test]
    fn wipes_cache_contents_under_temp_home() {
        let home = tempfile::tempdir().unwrap();
        let cache = home.path().join("Library/Caches/ms-playwright");
        std::fs::create_dir_all(cache.join("chromium")).unwrap();
        std::fs::write(cache.join("chromium/bin"), vec![0u8; 64 * 1024]).unwrap();
        let outcome = clean_area(&temp_ctx(home.path()), Area::Playwright, false, &mut |_| {});
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        assert!(outcome.freed() >= 64 * 1024);
        assert!(cache.is_dir());
        assert!(!cache.join("chromium").exists());
    }

    #[test]
    fn package_managers_use_their_own_command() {
        assert_eq!(tool_command(Area::Pnpm), Some(&["pnpm", "store", "prune"][..]));
        assert_eq!(tool_command(Area::Xcode), None);
        assert!(!wipes_paths(Area::Pnpm));
    }
}
