use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;

fn devclean(home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("devclean").unwrap();
    cmd.env("HOME", home).env("DEVCLEAN_CONFIG", home.join("config"));
    cmd
}

#[test]
fn version_names_the_author() {
    let home = tempfile::tempdir().unwrap();
    devclean(home.path())
        .arg("version")
        .assert()
        .success()
        .stdout(contains("devclean ").and(contains("built by Zain Zafar")));
}

#[test]
fn help_lists_areas() {
    let home = tempfile::tempdir().unwrap();
    devclean(home.path())
        .arg("help")
        .assert()
        .success()
        .stdout(contains("Areas: docker"));
}

#[test]
fn usage_errors_exit_2() {
    let home = tempfile::tempdir().unwrap();
    for args in [
        &["clean"][..],
        &["clean", "not-an-area"],
        &["clean", "a", "b"],
        &["bogus"],
        &["schedule", "sideways"],
    ] {
        devclean(home.path()).args(args).assert().code(2);
    }
}

#[test]
fn report_has_every_section() {
    let home = tempfile::tempdir().unwrap();
    devclean(home.path()).assert().success().stdout(
        contains("MEMORY")
            .and(contains("DISK"))
            .and(contains("CACHES"))
            .and(contains("worktrees")),
    );
}

#[test]
fn schedule_status_is_read_only() {
    let home = tempfile::tempdir().unwrap();
    devclean(home.path())
        .args(["schedule", "status"])
        .assert()
        .success()
        .stdout(contains("daily clean is"));
    assert!(!home.path().join("Library/LaunchAgents").exists());
}

#[test]
fn dry_run_keeps_files() {
    let home = tempfile::tempdir().unwrap();
    let cache = home.path().join("Library/Caches/CocoaPods");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::write(cache.join("pod"), "x").unwrap();
    devclean(home.path())
        .args(["clean", "cocoapods", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("would empty ~/Library/Caches/CocoaPods").and(contains("dry run: nothing changed")));
    assert!(cache.join("pod").exists());
}

#[test]
fn cleans_a_cache_inside_temp_home() {
    let home = tempfile::tempdir().unwrap();
    let cache = home.path().join("Library/Caches/CocoaPods");
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::write(cache.join("pod"), vec![0u8; 64 * 1024]).unwrap();
    devclean(home.path())
        .args(["clean", "cocoapods"])
        .assert()
        .success()
        .stdout(contains("freed about"));
    assert!(!cache.join("pod").exists());
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = std::process::Command::new("git")
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
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A home whose config lists one repo with one clean, fully pushed worktree at `<home>/wt`.
fn home_with_pushed_worktree() -> tempfile::TempDir {
    let home = tempfile::tempdir().unwrap();
    let root = home.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("remote.git")).unwrap();
    git(&root.join("remote.git"), &["init", "-q", "--bare"]);
    git(
        &root,
        &["clone", "-q", root.join("remote.git").to_str().unwrap(), "repo"],
    );
    git(&root.join("repo"), &["commit", "-q", "--allow-empty", "-m", "init"]);
    git(&root.join("repo"), &["push", "-q", "origin", "HEAD"]);
    git(
        &root.join("repo"),
        &["worktree", "add", "-q", "--detach", root.join("wt").to_str().unwrap()],
    );
    std::fs::write(root.join("config"), format!("{}\n", root.join("repo").display())).unwrap();
    home
}

#[test]
fn worktrees_are_kept_unless_the_user_says_yes() {
    let home = home_with_pushed_worktree();
    devclean(home.path())
        .args(["clean", "worktrees"])
        .write_stdin("")
        .assert()
        .success()
        .stdout(
            contains("would remove wt")
                .and(contains("Remove 1 worktree? [y/N]"))
                .and(contains("cancelled")),
        );
    assert!(home.path().join("wt").exists());
}

#[test]
fn worktrees_go_after_a_yes() {
    let home = home_with_pushed_worktree();
    devclean(home.path())
        .args(["clean", "worktrees"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(contains("removed wt"));
    assert!(!home.path().join("wt").exists());
}
