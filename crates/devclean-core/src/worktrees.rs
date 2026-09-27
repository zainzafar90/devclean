use std::path::{Path, PathBuf};

use crate::command::{run_ok, QUICK};
use crate::context::Context;

/// Repositories listed one per line in the config file; `#` comments, blanks and non-repos are skipped.
pub fn parse_config(text: &str, home: &Path) -> Vec<PathBuf> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| match line.strip_prefix('~') {
            Some(rest) => home.join(rest.trim_start_matches('/')),
            None => PathBuf::from(line),
        })
        .collect()
}

pub fn config_repos(ctx: &Context) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(&ctx.config_file) else {
        return Vec::new();
    };
    parse_config(&text, &ctx.home)
        .into_iter()
        .filter(|repo| repo.join(".git").is_dir())
        .collect()
}

/// Linked worktrees from `git worktree list --porcelain`; the first entry is the main checkout and is skipped.
pub fn parse_worktree_list(porcelain: &str) -> Vec<PathBuf> {
    porcelain
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .skip(1)
        .map(PathBuf::from)
        .collect()
}

pub fn worktree_paths(repo: &Path) -> Vec<PathBuf> {
    let repo = repo.to_string_lossy();
    run_ok("git", &["-C", &repo, "worktree", "list", "--porcelain"], QUICK)
        .map(|out| parse_worktree_list(&out))
        .unwrap_or_default()
}

pub fn all_worktree_paths(ctx: &Context) -> Vec<PathBuf> {
    config_repos(ctx).iter().flat_map(|repo| worktree_paths(repo)).collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Remove,
    Keep(String),
}

/// A worktree may go only when git confirms it has no changes and no commits missing from a remote.
pub fn decide(dirty: Option<bool>, unpushed: Option<u64>) -> Decision {
    match (dirty, unpushed) {
        (None, _) => Decision::Keep("git status failed".into()),
        (Some(true), _) => Decision::Keep("changes".into()),
        (_, None) => Decision::Keep("could not count unpushed commits".into()),
        (_, Some(0)) => Decision::Remove,
        (_, Some(count)) => Decision::Keep(format!("{count} unpushed commits")),
    }
}

pub fn inspect(worktree: &Path) -> Decision {
    let path = worktree.to_string_lossy();
    let dirty = run_ok("git", &["-C", &path, "status", "--porcelain"], QUICK)
        .ok()
        .map(|out| !out.trim().is_empty());
    let unpushed = run_ok(
        "git",
        &["-C", &path, "rev-list", "--count", "HEAD", "--not", "--remotes"],
        QUICK,
    )
    .ok()
    .and_then(|out| out.trim().parse().ok());
    decide(dirty, unpushed)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decisions() {
        assert_eq!(decide(Some(false), Some(0)), Decision::Remove);
        assert_eq!(decide(Some(true), Some(0)), Decision::Keep("changes".into()));
        assert_eq!(
            decide(Some(false), Some(2)),
            Decision::Keep("2 unpushed commits".into())
        );
        assert!(matches!(decide(None, Some(0)), Decision::Keep(_)));
        assert!(matches!(decide(Some(false), None), Decision::Keep(_)));
    }

    #[test]
    fn parses_config_and_porcelain() {
        let home = Path::new("/Users/dev");
        let repos = parse_config("# comment\n\n~/code/app\n/abs/repo\n  ", home);
        assert_eq!(
            repos,
            vec![PathBuf::from("/Users/dev/code/app"), PathBuf::from("/abs/repo")]
        );
        let porcelain = "worktree /r\nHEAD abc\nbranch refs/heads/main\n\nworktree /r-wt\nHEAD def\ndetached\n";
        assert_eq!(parse_worktree_list(porcelain), vec![PathBuf::from("/r-wt")]);
    }
}
