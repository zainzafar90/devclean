use std::sync::Mutex;
use std::time::{Duration, Instant};

use devclean_core::command::{run_ok, QUICK};
use tauri::window::{Effect, EffectState, EffectsBuilder};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewWindow};

pub const LABEL: &str = "panel";

/// When the panel last hid itself on blur; a tray click right after that is the same gesture, not a reopen.
#[derive(Default)]
pub struct LastHidden(Mutex<Option<Instant>>);

impl LastHidden {
    pub fn mark(&self) {
        if let Ok(mut last) = self.0.lock() {
            *last = Some(Instant::now());
        }
    }

    fn just_now(&self) -> bool {
        self.0
            .lock()
            .map(|last| last.is_some_and(|at| at.elapsed() < Duration::from_millis(300)))
            .unwrap_or(false)
    }
}

/// Liquid Glass on macOS 26 and later, the popover material before it, with each system's menu corner radius.
pub fn apply_material(panel: &WebviewWindow) -> tauri::Result<()> {
    let radius = if macos_major_version() >= 26 { 18.0 } else { 12.0 };
    panel.set_effects(
        EffectsBuilder::new()
            .effects([Effect::LiquidGlassRegular, Effect::Popover])
            .state(EffectState::Active)
            .radius(radius)
            .build(),
    )
}

fn macos_major_version() -> u32 {
    run_ok("sw_vers", &["-productVersion"], QUICK)
        .ok()
        .and_then(|version| version.trim().split('.').next()?.parse().ok())
        .unwrap_or(0)
}

pub fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(LABEL)
}

pub fn toggle(app: &AppHandle, click: PhysicalPosition<f64>) {
    let Some(panel) = window(app) else { return };
    if panel.is_visible().unwrap_or(false) {
        let _ = panel.hide();
        return;
    }
    if app.state::<LastHidden>().just_now() {
        return;
    }
    show_near(&panel, click);
}

/// Shows the panel just under the menu bar, centred on `anchor` and kept on its screen.
/// macOS 26 hosts status items in Control Center, so the tray rect is unreliable; the click point is not.
pub fn show_near(panel: &WebviewWindow, anchor: PhysicalPosition<f64>) {
    let monitor = panel
        .monitor_from_point(anchor.x, anchor.y)
        .ok()
        .flatten()
        .or_else(|| panel.primary_monitor().ok().flatten());
    if let Some(monitor) = monitor {
        let area = monitor.work_area();
        let width = panel.outer_size().map(|size| size.width as i32).unwrap_or(0);
        let left = area.position.x;
        let right = (left + area.size.width as i32 - width).max(left);
        let x = (anchor.x as i32 - width / 2).clamp(left, right);
        let _ = panel.set_position(PhysicalPosition::new(x, area.position.y + 4));
    }
    let _ = panel.show();
    let _ = panel.set_focus();
}

/// Top-right corner of the primary screen, where the menu-bar icon usually sits.
pub fn show_at_top_right(panel: &WebviewWindow) {
    if let Ok(Some(monitor)) = panel.primary_monitor() {
        let corner = PhysicalPosition::new(
            (monitor.position().x + monitor.size().width as i32) as f64,
            monitor.position().y as f64,
        );
        show_near(panel, corner);
    }
}
