#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::app::Application;

mod app;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(Application::default()), options);
}
