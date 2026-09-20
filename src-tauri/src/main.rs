// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod commands;
mod error;
mod model;
mod open;
mod ops;
mod path;
mod read;
mod size;
mod util;
mod watch;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::ping,])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
