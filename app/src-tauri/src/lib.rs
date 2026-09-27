mod commands;
mod panel;
mod tray;

use devclean_core::Context;
use tauri::{ActivationPolicy, Manager, Theme, WindowEvent};

pub fn run_scheduled() -> i32 {
    let result = Context::from_env().map_err(|err| err.to_string()).and_then(|ctx| {
        devclean_core::schedule::scheduled_run(&ctx, &mut std::io::stdout()).map_err(|e| e.to_string())
    });
    match result {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(err) => {
            eprintln!("devclean: {err}");
            1
        }
    }
}

/// `--open` shows the panel at launch and `--theme light|dark` pins the appearance (used for screenshots).
fn launch_theme() -> Option<Theme> {
    let args: Vec<String> = std::env::args().collect();
    let value = args
        .iter()
        .position(|arg| arg == "--theme")
        .and_then(|i| args.get(i + 1))?;
    match value.as_str() {
        "dark" => Some(Theme::Dark),
        "light" => Some(Theme::Light),
        _ => None,
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(commands::CleanLock::default())
        .manage(panel::LastHidden::default())
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::area_sizes,
            commands::areas,
            commands::area_size,
            commands::clean,
            commands::schedule_status,
            commands::set_schedule,
            commands::app_info,
            commands::open_repo,
            commands::quit,
        ])
        .setup(|app| {
            app.set_activation_policy(ActivationPolicy::Accessory);
            let tray = tray::build(app.handle())?;
            if let Some(panel) = panel::window(app.handle()) {
                panel::apply_material(&panel)?;
                if let Some(theme) = launch_theme() {
                    panel.set_theme(Some(theme))?;
                }
                if std::env::args().any(|arg| arg == "--open") {
                    panel::show_at_top_right(&panel);
                }
            }
            tray::keep_title_fresh(tray);
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == panel::LABEL {
                if let WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                    window.app_handle().state::<panel::LastHidden>().mark();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("devclean app failed to start");
}
