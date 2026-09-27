use std::path::{Path, PathBuf};

use crate::command::{has, run_ok, LONG, QUICK};
use crate::context::Context;
use crate::worktrees::{config_repos, inspect, worktree_paths, Decision};

/// Entries git ignores (`.env`, `node_modules/`); they are deleted with the worktree. `None` when git could not list them.
pub fn ignored_entries(worktree: &Path) -> Option<Vec<String>> {
    let path = worktree.to_string_lossy();
    let out = run_ok("git", &["-C", &path, "status", "--porcelain", "--ignored"], QUICK).ok()?;
    Some(
        out.lines()
            .filter_map(|line| line.strip_prefix("!! "))
            .map(str::to_string)
            .collect(),
    )
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub repo: PathBuf,
    pub path: PathBuf,
    pub decision: Decision,
    pub ignored: Option<Vec<String>>,
}

impl Candidate {
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn removable(&self) -> bool {
        self.decision == Decision::Remove
    }
}

/// Fetches each configured repo and inspects its linked worktrees. Removes nothing.
pub fn plan(ctx: &Context, on_line: &mut dyn FnMut(&str)) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    for repo in config_repos(ctx) {
        let repo_arg = repo.to_string_lossy();
        if let Err(err) = run_ok("git", &["-C", &repo_arg, "fetch", "-q", "origin"], LONG) {
            on_line(&format!(
                "fetch failed for {}, using known remote state ({err})",
                ctx.tilde(&repo)
            ));
        }
        for path in worktree_paths(&repo) {
            let decision = inspect(&path);
            let ignored = if decision == Decision::Remove {
                ignored_entries(&path)
            } else {
                Some(Vec::new())
            };
            candidates.push(Candidate {
                repo: repo.clone(),
                path,
                decision,
                ignored,
            });
        }
    }
    candidates
}

const IGNORED_SHOWN: usize = 5;

fn ignored_note(ignored: &Option<Vec<String>>) -> String {
    match ignored {
        None => " (could not list git-ignored files; any inside go too)".to_string(),
        Some(entries) if entries.is_empty() => String::new(),
        Some(entries) => {
            let shown = entries
                .iter()
                .take(IGNORED_SHOWN)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let more = entries.len().saturating_sub(IGNORED_SHOWN);
            let tail = if more > 0 {
                format!(" and {more} more")
            } else {
                String::new()
            };
            format!(" (git-ignored files go too: {shown}{tail})")
        }
    }
}

/// One preview line: what would happen to this worktree and why.
pub fn describe(candidate: &Candidate) -> String {
    match &candidate.decision {
        Decision::Keep(reason) => format!("kept    {} ({reason})", candidate.name()),
        Decision::Remove => format!("would remove {}{}", candidate.name(), ignored_note(&candidate.ignored)),
    }
}

/// Removes the removable candidates, re-checking each one first so work added after the preview is kept.
pub fn remove(candidates: &[Candidate], on_line: &mut dyn FnMut(&str)) -> Vec<String> {
    let mut failures = Vec::new();
    for candidate in candidates.iter().filter(|c| c.removable()) {
        let name = candidate.name();
        if let Decision::Keep(reason) = inspect(&candidate.path) {
            on_line(&format!("kept    {name} ({reason})"));
            continue;
        }
        let repo = candidate.repo.to_string_lossy();
        let path = candidate.path.to_string_lossy();
        match run_ok("git", &["-C", &repo, "worktree", "remove", "--", &path], LONG) {
            Ok(_) => on_line(&format!("removed {name}")),
            Err(err) => {
                on_line(&format!("kept    {name} (git refused)"));
                failures.push(err.to_string());
            }
        }
    }
    let mut repos: Vec<&PathBuf> = candidates.iter().map(|c| &c.repo).collect();
    repos.dedup();
    for repo in repos {
        if let Err(err) = run_ok("git", &["-C", &repo.to_string_lossy(), "worktree", "prune"], QUICK) {
            failures.push(err.to_string());
        }
    }
    failures
}

pub fn clean(ctx: &Context, dry_run: bool, on_line: &mut dyn FnMut(&str)) -> Vec<String> {
    if !has("git") {
        on_line("git not installed, skipped");
        return Vec::new();
    }
    let candidates = plan(ctx, on_line);
    for candidate in candidates.iter().filter(|c| dry_run || !c.removable()) {
        on_line(&describe(candidate));
    }
    if dry_run {
        return Vec::new();
    }
    remove(&candidates, on_line)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.test")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.test")
            .output()
            .unwrap();
        assert!(
            status.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }

    #[test]
    fn removes_only_clean_pushed_worktrees() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let remote = root.join("remote.git");
        let repo = root.join("repo");
        std::fs::create_dir_all(&remote).unwrap();
        git(&remote, &["init", "-q", "--bare"]);
        git(&root, &["clone", "-q", remote.to_str().unwrap(), "repo"]);
        git(&repo, &["commit", "-q", "--allow-empty", "-m", "init"]);
        git(&repo, &["push", "-q", "origin", "HEAD"]);
        for name in ["pushed", "dirty", "ahead"] {
            git(
                &repo,
                &["worktree", "add", "-q", "--detach", root.join(name).to_str().unwrap()],
            );
        }
        std::fs::write(root.join("dirty/new.txt"), "x").unwrap();
        git(&root.join("ahead"), &["commit", "-q", "--allow-empty", "-m", "local"]);

        let config = root.join("config");
        std::fs::write(&config, format!("{}\n", repo.display())).unwrap();
        let ctx = Context {
            home: root.clone(),
            config_file: config,
        };

        let mut lines = Vec::new();
        let failures = clean(&ctx, true, &mut |line| lines.push(line.to_string()));
        assert!(failures.is_empty(), "{failures:?}");
        assert!(lines.contains(&"would remove pushed".to_string()), "{lines:?}");
        assert!(root.join("pushed").exists());

        lines.clear();
        let failures = clean(&ctx, false, &mut |line| lines.push(line.to_string()));
        assert!(failures.is_empty(), "{failures:?}");
        assert!(lines.contains(&"removed pushed".to_string()), "{lines:?}");
        assert!(lines.contains(&"kept    dirty (changes)".to_string()), "{lines:?}");
        assert!(
            lines.contains(&"kept    ahead (1 unpushed commits)".to_string()),
            "{lines:?}"
        );
        assert!(!root.join("pushed").exists());
        assert!(root.join("dirty/new.txt").exists());
        assert!(root.join("ahead").exists());
    }

    fn pushed_repo(root: &Path) -> (PathBuf, Context) {
        let remote = root.join("remote.git");
        let repo = root.join("repo");
        std::fs::create_dir_all(&remote).unwrap();
        git(&remote, &["init", "-q", "--bare"]);
        git(root, &["clone", "-q", remote.to_str().unwrap(), "repo"]);
        std::fs::write(repo.join(".gitignore"), ".env\n").unwrap();
        git(&repo, &["add", ".gitignore"]);
        git(&repo, &["commit", "-q", "-m", "init"]);
        git(&repo, &["push", "-q", "origin", "HEAD"]);
        let config = root.join("config");
        std::fs::write(&config, format!("{}\n", repo.display())).unwrap();
        let ctx = Context {
            home: root.to_path_buf(),
            config_file: config,
        };
        (repo, ctx)
    }

    #[test]
    fn preview_names_git_ignored_files_that_go_too() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let (repo, ctx) = pushed_repo(&root);
        git(
            &repo,
            &["worktree", "add", "-q", "--detach", root.join("wt").to_str().unwrap()],
        );
        std::fs::write(root.join("wt/.env"), "SECRET=1").unwrap();

        let candidates = plan(&ctx, &mut |_| {});
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].removable());
        assert_eq!(
            describe(&candidates[0]),
            "would remove wt (git-ignored files go too: .env)"
        );
    }

    #[test]
    fn rechecks_each_worktree_right_before_removing_it() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let (repo, ctx) = pushed_repo(&root);
        git(
            &repo,
            &["worktree", "add", "-q", "--detach", root.join("late").to_str().unwrap()],
        );

        let candidates = plan(&ctx, &mut |_| {});
        assert!(candidates[0].removable());
        git(
            &root.join("late"),
            &["commit", "-q", "--allow-empty", "-m", "made after the preview"],
        );

        let mut lines = Vec::new();
        let failures = remove(&candidates, &mut |line| lines.push(line.to_string()));
        assert!(failures.is_empty(), "{failures:?}");
        assert!(
            lines.contains(&"kept    late (1 unpushed commits)".to_string()),
            "{lines:?}"
        );
        assert!(root.join("late").exists());
    }
}
