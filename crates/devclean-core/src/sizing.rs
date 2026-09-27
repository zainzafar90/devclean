use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

/// Allocated bytes under `path` like `du -sk`: symlinks are not followed and unreadable entries count as zero.
pub fn disk_usage(path: &Path) -> u64 {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return 0;
    };
    let own = meta.blocks() * 512;
    if !meta.is_dir() {
        return own;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return own;
    };
    let children: Vec<PathBuf> = entries.filter_map(|entry| entry.ok().map(|e| e.path())).collect();
    own + children.par_iter().map(|child| disk_usage(child)).sum::<u64>()
}

pub fn total_usage(paths: &[PathBuf]) -> u64 {
    paths.par_iter().map(|path| disk_usage(path)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_nested_files_and_skips_missing() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(dir.path().join("a/b/file"), vec![1u8; 64 * 1024]).unwrap();
        fs::write(dir.path().join("top"), vec![1u8; 8 * 1024]).unwrap();
        let size = disk_usage(dir.path());
        assert!(size >= 72 * 1024, "size {size}");
        assert_eq!(disk_usage(&dir.path().join("missing")), 0);
    }

    #[test]
    fn does_not_follow_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let big = dir.path().join("big");
        fs::create_dir(&big).unwrap();
        fs::write(big.join("file"), vec![1u8; 256 * 1024]).unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&big, &link).unwrap();
        assert!(disk_usage(&link) < 16 * 1024);
    }

    #[test]
    fn unreadable_directory_counts_as_zero() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let locked = dir.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::write(locked.join("file"), vec![1u8; 256 * 1024]).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        let size = disk_usage(dir.path());
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(size < 256 * 1024);
    }
}
