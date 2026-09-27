pub mod areas;
pub mod clean;
pub mod clock;
pub mod command;
pub mod context;
pub mod disk;
pub mod docker;
pub mod docker_cleanup;
pub mod error;
pub mod format;
pub mod memory;
pub mod path_guard;
pub mod processes;
pub mod schedule;
pub mod sizing;
pub mod snapshot;
mod sysctl;
pub mod worktree_cleanup;
pub mod worktrees;

pub use areas::{area_sizes, Area, AreaSize};
pub use clean::{clean_area, CleanOutcome};
pub use context::Context;
pub use error::{CoreError, Result};
pub use format::human;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "Zain Zafar";
