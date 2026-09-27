use serde::Serialize;
use sysinfo::System;

use crate::sysctl;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Pressure {
    Normal,
    Warning,
    Critical,
    Unknown,
}

impl Pressure {
    pub fn from_level(level: Option<i32>) -> Self {
        match level {
            Some(1) => Self::Normal,
            Some(2) => Self::Warning,
            Some(4) => Self::Critical,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    /// The "memory free percentage" macOS itself reports (`memory_pressure`).
    pub free_percent: Option<u8>,
    pub pressure: Pressure,
    pub swap_total: u64,
    pub swap_used: u64,
}

impl MemoryInfo {
    pub fn is_low(&self) -> bool {
        self.free_percent.is_some_and(|free| free < 20)
    }
}

pub fn memory_info() -> MemoryInfo {
    let mut system = System::new();
    system.refresh_memory();
    let free_percent = sysctl::read_i32(c"kern.memorystatus_level")
        .and_then(|level| u8::try_from(level).ok())
        .filter(|level| *level <= 100);
    MemoryInfo {
        total: system.total_memory(),
        used: system.used_memory(),
        available: system.available_memory(),
        free_percent,
        pressure: Pressure::from_level(sysctl::read_i32(c"kern.memorystatus_vm_pressure_level")),
        swap_total: system.total_swap(),
        swap_used: system.used_swap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_live_memory() {
        let info = memory_info();
        assert!(info.total > 0);
        assert!(info.free_percent.is_some());
        assert_ne!(info.pressure, Pressure::Unknown);
    }

    #[test]
    fn maps_pressure_levels() {
        assert_eq!(Pressure::from_level(Some(4)), Pressure::Critical);
        assert_eq!(Pressure::from_level(None), Pressure::Unknown);
    }
}
