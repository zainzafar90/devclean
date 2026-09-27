use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::context::Context;
use crate::error::CoreError;
use crate::{docker, sizing, worktrees};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Area {
    Docker,
    Pnpm,
    Uv,
    Npm,
    Yarn,
    Brew,
    Pip,
    Xcode,
    Simulators,
    Playwright,
    Cocoapods,
    Worktrees,
}

impl Area {
    pub const ALL: [Area; 12] = [
        Area::Docker,
        Area::Pnpm,
        Area::Uv,
        Area::Npm,
        Area::Yarn,
        Area::Brew,
        Area::Pip,
        Area::Xcode,
        Area::Simulators,
        Area::Playwright,
        Area::Cocoapods,
        Area::Worktrees,
    ];

    pub fn safe() -> impl Iterator<Item = Area> {
        Self::ALL.into_iter().filter(|area| area.is_safe())
    }

    pub fn name(self) -> &'static str {
        match self {
            Area::Docker => "docker",
            Area::Pnpm => "pnpm",
            Area::Uv => "uv",
            Area::Npm => "npm",
            Area::Yarn => "yarn",
            Area::Brew => "brew",
            Area::Pip => "pip",
            Area::Xcode => "xcode",
            Area::Simulators => "simulators",
            Area::Playwright => "playwright",
            Area::Cocoapods => "cocoapods",
            Area::Worktrees => "worktrees",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Area::Docker => "Docker: unused images, volumes, build cache, stale test containers and builders",
            Area::Pnpm => "pnpm store (unreferenced packages)",
            Area::Uv => "uv cache",
            Area::Npm => "npm cache",
            Area::Yarn => "Yarn cache",
            Area::Brew => "Homebrew downloads and old versions",
            Area::Pip => "pip cache",
            Area::Xcode => "Xcode DerivedData",
            Area::Simulators => "iOS simulator caches and unavailable devices",
            Area::Playwright => "Playwright browsers (re-download: npx playwright install)",
            Area::Cocoapods => "CocoaPods cache",
            Area::Worktrees => "git worktrees that are merged and have no changes",
        }
    }

    /// Safe areas rebuild themselves on demand; the others cost a re-download or remove work trees.
    pub fn is_safe(self) -> bool {
        !matches!(self, Area::Playwright | Area::Cocoapods | Area::Worktrees)
    }

    pub fn paths(self, home: &Path) -> Vec<PathBuf> {
        let relative: &[&str] = match self {
            Area::Pnpm => &["Library/pnpm/store", "Library/Caches/pnpm", ".local/share/pnpm/store"],
            Area::Uv => &[".cache/uv", "Library/Caches/uv"],
            Area::Npm => &[".npm/_cacache"],
            Area::Yarn => &["Library/Caches/Yarn"],
            Area::Brew => &["Library/Caches/Homebrew"],
            Area::Pip => &["Library/Caches/pip"],
            Area::Xcode => &["Library/Developer/Xcode/DerivedData"],
            Area::Simulators => &["Library/Developer/CoreSimulator/Caches"],
            Area::Playwright => &["Library/Caches/ms-playwright"],
            Area::Cocoapods => &["Library/Caches/CocoaPods"],
            Area::Docker | Area::Worktrees => &[],
        };
        relative.iter().map(|path| home.join(path)).collect()
    }
}

impl fmt::Display for Area {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Area {
    type Err = CoreError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|area| area.name() == name)
            .ok_or_else(|| CoreError::UnknownArea(name.to_string()))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AreaSize {
    pub area: Area,
    pub label: &'static str,
    pub safe: bool,
    pub bytes: u64,
}

pub fn area_size(ctx: &Context, area: Area) -> u64 {
    match area {
        Area::Docker => docker::reclaimable(),
        Area::Worktrees => sizing::total_usage(&worktrees::all_worktree_paths(ctx)),
        _ => sizing::total_usage(&area.paths(&ctx.home)),
    }
}

/// Every area's size, measured in parallel, in registry order.
pub fn area_sizes(ctx: &Context) -> Vec<AreaSize> {
    Area::ALL
        .par_iter()
        .map(|&area| AreaSize {
            area,
            label: area.label(),
            safe: area.is_safe(),
            bytes: area_size(ctx, area),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_matches_bash_version() {
        let names: Vec<&str> = Area::ALL.iter().map(|area| area.name()).collect();
        assert_eq!(
            names.join(" "),
            "docker pnpm uv npm yarn brew pip xcode simulators playwright cocoapods worktrees"
        );
        let safe: Vec<&str> = Area::safe().map(Area::name).collect();
        assert_eq!(safe.join(" "), "docker pnpm uv npm yarn brew pip xcode simulators");
    }

    #[test]
    fn parses_names() {
        assert_eq!("xcode".parse::<Area>().unwrap(), Area::Xcode);
        assert!(matches!("safe".parse::<Area>(), Err(CoreError::UnknownArea(_))));
    }

    #[test]
    fn wiped_paths_stay_under_library_or_cache() {
        let home = Path::new("/Users/dev");
        for area in [Area::Xcode, Area::Simulators, Area::Playwright, Area::Cocoapods] {
            for path in area.paths(home) {
                assert!(crate::path_guard::is_inside_deletable_root(&path, home), "{path:?}");
            }
        }
    }

    #[test]
    fn sizes_areas_under_a_temp_home() {
        let home = tempfile::tempdir().unwrap();
        let derived = home.path().join("Library/Developer/Xcode/DerivedData/App");
        std::fs::create_dir_all(&derived).unwrap();
        std::fs::write(derived.join("build"), vec![0u8; 128 * 1024]).unwrap();
        let ctx = Context {
            home: home.path().to_path_buf(),
            config_file: home.path().join("none"),
        };
        assert!(area_size(&ctx, Area::Xcode) >= 128 * 1024);
        assert_eq!(area_size(&ctx, Area::Pip), 0);
        assert_eq!(area_size(&ctx, Area::Worktrees), 0);
    }
}
