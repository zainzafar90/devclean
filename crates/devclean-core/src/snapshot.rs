use serde::Serialize;

use crate::context::Context;
use crate::disk::{disk_info, DiskInfo};
use crate::error::Result;
use crate::format::human;
use crate::memory::{memory_info, MemoryInfo};
use crate::processes::{top_processes, ProcessUsage};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub memory: MemoryInfo,
    pub top_apps: Vec<ProcessUsage>,
    pub disk: DiskInfo,
}

/// Memory, the heaviest processes and free disk space; cheap enough to poll every few seconds.
pub fn snapshot(ctx: &Context, top: usize) -> Result<Snapshot> {
    Ok(Snapshot {
        memory: memory_info(),
        top_apps: top_processes(top)?,
        disk: disk_info(&ctx.home)?,
    })
}

/// Menu-bar text: memory free and disk free, prefixed with ⚠ when either is low.
pub fn menu_bar_title(memory: &MemoryInfo, disk: Option<&DiskInfo>) -> String {
    let memory_text = match memory.free_percent {
        Some(free) => format!("{free}%"),
        None => "?%".to_string(),
    };
    let disk_text = disk.map(|disk| human(disk.free)).unwrap_or_else(|| "?".to_string());
    let low = memory.is_low() || disk.is_some_and(DiskInfo::is_low);
    let warn = if low { "⚠ " } else { "" };
    format!("{warn}{memory_text} · {disk_text}")
}

pub fn current_menu_bar_title(ctx: &Context) -> String {
    menu_bar_title(&memory_info(), disk_info(&ctx.home).ok().as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Pressure;

    fn memory(free_percent: Option<u8>) -> MemoryInfo {
        MemoryInfo {
            total: 0,
            used: 0,
            available: 0,
            free_percent,
            pressure: Pressure::Normal,
            swap_total: 0,
            swap_used: 0,
        }
    }

    fn disk(free_percent: u8) -> DiskInfo {
        DiskInfo {
            total: 100 * 1024u64.pow(3),
            free: u64::from(free_percent) * 1024u64.pow(3),
            free_percent,
        }
    }

    #[test]
    fn title_shows_memory_and_disk() {
        assert_eq!(menu_bar_title(&memory(Some(47)), Some(&disk(50))), "47% · 50.0 GB");
    }

    #[test]
    fn title_warns_when_memory_or_disk_is_low() {
        assert_eq!(menu_bar_title(&memory(Some(19)), Some(&disk(50))), "⚠ 19% · 50.0 GB");
        assert_eq!(menu_bar_title(&memory(Some(47)), Some(&disk(9))), "⚠ 47% · 9.0 GB");
        assert_eq!(menu_bar_title(&memory(None), None), "?% · ?");
    }
}
