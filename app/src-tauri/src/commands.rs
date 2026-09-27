use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use devclean_core::schedule::{self, ScheduleStatus};
use devclean_core::snapshot::Snapshot;
use devclean_core::{Area, AreaSize, CleanOutcome, Context, CoreError};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

pub const REPO_URL: &str = "https://github.com/zainzafar90/devclean";
const TOP_APPS: usize = 6;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error("a clean is already running")]
    Busy,
    #[error("background task failed: {0}")]
    Task(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

type CommandResult<T> = Result<T, AppError>;

#[derive(Default)]
pub struct CleanLock(AtomicBool);

struct CleanGuard<'a>(&'a AtomicBool);

impl Drop for CleanGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl CleanLock {
    fn acquire(&self) -> CommandResult<CleanGuard<'_>> {
        self.0
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| AppError::Busy)?;
        Ok(CleanGuard(&self.0))
    }
}

async fn blocking<T: Send + 'static>(job: impl FnOnce() -> CommandResult<T> + Send + 'static) -> CommandResult<T> {
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|err| AppError::Task(err.to_string()))?
}

#[tauri::command]
pub async fn snapshot() -> CommandResult<Snapshot> {
    blocking(|| Ok(devclean_core::snapshot::snapshot(&Context::from_env()?, TOP_APPS)?)).await
}

#[tauri::command]
pub async fn area_sizes() -> CommandResult<Vec<AreaSize>> {
    blocking(|| Ok(devclean_core::area_sizes(&Context::from_env()?))).await
}

#[derive(Serialize)]
pub struct AreaInfo {
    area: Area,
    label: &'static str,
    safe: bool,
}

/// The area registry without sizes, so the panel can list every row before measuring.
#[tauri::command]
pub fn areas() -> Vec<AreaInfo> {
    Area::ALL
        .into_iter()
        .map(|area| AreaInfo {
            area,
            label: area.label(),
            safe: area.is_safe(),
        })
        .collect()
}

#[tauri::command]
pub async fn area_size(area: Area) -> CommandResult<u64> {
    blocking(move || Ok(devclean_core::areas::area_size(&Context::from_env()?, area))).await
}

#[derive(Clone, Serialize)]
struct CleanProgress {
    area: Area,
    line: String,
}

#[tauri::command]
pub async fn clean(app: AppHandle, lock: State<'_, CleanLock>, areas: Vec<Area>) -> CommandResult<Vec<CleanOutcome>> {
    let _guard = lock.acquire()?;
    blocking(move || {
        let ctx = Context::from_env()?;
        let outcomes = areas
            .into_iter()
            .map(|area| {
                let mut emit = |line: &str| {
                    let _ = app.emit(
                        "clean-progress",
                        CleanProgress {
                            area,
                            line: line.to_string(),
                        },
                    );
                };
                emit(&format!("cleaning {area}"));
                devclean_core::clean_area(&ctx, area, false, &mut emit)
            })
            .collect();
        Ok(outcomes)
    })
    .await
}

#[tauri::command]
pub async fn schedule_status() -> CommandResult<ScheduleStatus> {
    blocking(|| Ok(schedule::status(&Context::from_env()?))).await
}

#[tauri::command]
pub async fn set_schedule(enabled: bool) -> CommandResult<ScheduleStatus> {
    blocking(move || {
        let ctx = Context::from_env()?;
        if enabled {
            schedule::enable(&ctx, &schedule_program(&ctx)?)?;
        } else {
            schedule::disable(&ctx)?;
        }
        Ok(schedule::status(&ctx))
    })
    .await
}

/// The installed CLI when present, otherwise this app binary, which also understands `scheduled`.
fn schedule_program(ctx: &Context) -> CommandResult<PathBuf> {
    let cli = ctx.home.join(".local/bin/devclean");
    if cli.is_file() {
        return Ok(cli);
    }
    std::env::current_exe().map_err(|err| AppError::Core(CoreError::io("current executable", err)))
}

#[derive(Serialize)]
pub struct AppInfo {
    name: &'static str,
    version: &'static str,
    author: &'static str,
    repo: &'static str,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: "devclean",
        version: devclean_core::VERSION,
        author: devclean_core::AUTHOR,
        repo: REPO_URL,
    }
}

#[tauri::command]
pub fn open_repo() -> CommandResult<()> {
    devclean_core::command::run_ok("open", &[REPO_URL], devclean_core::command::QUICK)?;
    Ok(())
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}
