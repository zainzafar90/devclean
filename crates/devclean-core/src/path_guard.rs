use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::error::{CoreError, Result};

const DELETABLE_ROOTS: [&str; 2] = ["Library", ".cache"];

/// Lexical check: an absolute path strictly inside ~/Library or ~/.cache, with no `..` parts.
pub fn is_inside_deletable_root(path: &Path, home: &Path) -> bool {
    if !path.is_absolute() || !home.is_absolute() || home.parent().is_none() {
        return false;
    }
    let plain = path
        .components()
        .all(|part| matches!(part, Component::RootDir | Component::Normal(_)));
    plain
        && DELETABLE_ROOTS.iter().any(|root| {
            let root = home.join(root);
            path.starts_with(&root) && path != root
        })
}

/// Returns the path when it may be deleted: lexically allowed, not a symlink, and its real location is still allowed.
pub fn ensure_deletable(path: &Path, home: &Path) -> Result<PathBuf> {
    if !is_inside_deletable_root(path, home) {
        return Err(CoreError::UnsafePath(path.to_path_buf()));
    }
    let meta = fs::symlink_metadata(path).map_err(|err| CoreError::io(path, err))?;
    if meta.file_type().is_symlink() {
        return Err(CoreError::UnsafePath(path.to_path_buf()));
    }
    let real = path.canonicalize().map_err(|err| CoreError::io(path, err))?;
    let real_home = home.canonicalize().map_err(|err| CoreError::io(home, err))?;
    if !is_inside_deletable_root(&real, &real_home) {
        return Err(CoreError::UnsafePath(path.to_path_buf()));
    }
    Ok(real)
}

/// Deletes the contents of an allowed directory, keeping the directory itself. Returns one message per failure.
pub fn wipe_contents(path: &Path, home: &Path) -> Result<Vec<String>> {
    let dir = ensure_deletable(path, home)?;
    let entries = fs::read_dir(&dir).map_err(|err| CoreError::io(&dir, err))?;
    let mut failures = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| CoreError::io(&dir, err))?;
        let child = entry.path();
        let removed = match entry.file_type() {
            Ok(kind) if kind.is_dir() => fs::remove_dir_all(&child),
            Ok(_) => fs::remove_file(&child),
            Err(err) => Err(err),
        };
        if let Err(err) = removed {
            failures.push(format!("{}: {err}", child.display()));
        }
    }
    Ok(failures)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home() -> PathBuf {
        PathBuf::from("/Users/dev")
    }

    #[test]
    fn allows_only_strictly_inside_library_and_cache() {
        let home = home();
        assert!(is_inside_deletable_root(
            Path::new("/Users/dev/Library/Caches/pip"),
            &home
        ));
        assert!(is_inside_deletable_root(Path::new("/Users/dev/.cache/uv"), &home));
        for rejected in [
            "/Users/dev/Library",
            "/Users/dev/.cache",
            "/Users/dev/Library/../Documents",
            "/Users/dev/Libraryx/foo",
            "/Users/dev/Documents",
            "/Users/dev",
            "/Library/Caches",
            "/",
            "Library/Caches",
        ] {
            assert!(!is_inside_deletable_root(Path::new(rejected), &home), "{rejected}");
        }
        assert!(!is_inside_deletable_root(Path::new("/Library/x"), Path::new("/")));
    }

    #[test]
    fn rejects_symlinks_that_escape() {
        let home = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("keep"), "x").unwrap();
        fs::create_dir_all(home.path().join("Library/Caches")).unwrap();
        let link = home.path().join("Library/Caches/escape");
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        assert!(matches!(
            ensure_deletable(&link, home.path()),
            Err(CoreError::UnsafePath(_))
        ));
        let linked_library = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), linked_library.path().join("Library")).unwrap();
        let via = linked_library.path().join("Library");
        assert!(ensure_deletable(&via.join("keep"), linked_library.path()).is_err());
        assert!(outside.path().join("keep").exists());
    }

    #[test]
    fn wipes_contents_but_not_symlink_targets() {
        let home = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("keep"), "x").unwrap();
        let cache = home.path().join("Library/Caches/tool");
        fs::create_dir_all(cache.join("nested")).unwrap();
        fs::write(cache.join("nested/file"), "x").unwrap();
        fs::write(cache.join("file"), "x").unwrap();
        std::os::unix::fs::symlink(outside.path(), cache.join("link")).unwrap();
        let failures = wipe_contents(&cache, home.path()).unwrap();
        assert!(failures.is_empty(), "{failures:?}");
        assert!(cache.is_dir());
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 0);
        assert!(outside.path().join("keep").exists());
    }

    #[test]
    fn refuses_to_wipe_outside_roots() {
        let home = tempfile::tempdir().unwrap();
        let docs = home.path().join("Documents");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("file"), "x").unwrap();
        assert!(wipe_contents(&docs, home.path()).is_err());
        assert!(docs.join("file").exists());
    }
}
