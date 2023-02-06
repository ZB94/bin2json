#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::app::Application;

mod app;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|ctx| Box::new(Application::new(ctx))),
    );
}
