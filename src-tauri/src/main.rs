// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 对command单独管理
mod command;
mod utils;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        .invoke_handler(tauri::generate_handler![
            command::live::get_live_html,
            command::live::greet_you,
            command::live::open_window,
            command::cookie::save_cookies,
            command::cookie::load_cookies,
            command::cookie::clear_cookies,
            command::cookie::open_login_page
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
