#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().nth(1).as_deref() == Some("scheduled") {
        std::process::exit(devclean_app_lib::run_scheduled());
    }
    devclean_app_lib::run();
}
