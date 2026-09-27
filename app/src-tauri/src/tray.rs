use std::time::Duration;

use devclean_core::snapshot::current_menu_bar_title;
use devclean_core::Context;
use tauri::image::Image;
use tauri::menu::{AboutMetadataBuilder, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

use crate::panel;

const REFRESH: Duration = Duration::from_secs(5);

fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let about = AboutMetadataBuilder::new()
        .name(Some("devclean"))
        .version(Some(devclean_core::VERSION))
        .authors(Some(vec![devclean_core::AUTHOR.to_string()]))
        .copyright(Some("© 2026 Zain Zafar"))
        .comments(Some("Built by Zain Zafar"))
        .website(Some(crate::commands::REPO_URL))
        .icon(app.default_window_icon().cloned())
        .build();
    Menu::with_items(
        app,
        &[
            &PredefinedMenuItem::about(app, Some("About devclean"), Some(about))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Quit devclean", true, Some("Cmd+Q"))?,
        ],
    )
}

pub fn build(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon@2x.png"))?;
    TrayIconBuilder::with_id("devclean")
        .icon(icon)
        .icon_as_template(true)
        .title("…")
        .tooltip("devclean")
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                panel::toggle(tray.app_handle(), position);
            }
        })
        .build(app)
}

/// Keeps the menu-bar title current for the life of the app.
pub fn keep_title_fresh(tray: TrayIcon) {
    std::thread::spawn(move || loop {
        if let Ok(ctx) = Context::from_env() {
            let _ = tray.set_title(Some(current_menu_bar_title(&ctx)));
        }
        std::thread::sleep(REFRESH);
    });
}
