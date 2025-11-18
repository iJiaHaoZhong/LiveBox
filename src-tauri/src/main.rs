// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

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
        .on_window_event(|event| {
            // 当主窗口被关闭时，如果登录窗口在运行，则隐藏主窗口而不是退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                let window = event.window();

                // 如果这不是登录窗口，检查是否有登录窗口正在运行
                if window.label() != "douyinLogin" {
                    if let Some(_login_window) = window.app_handle().get_window("douyinLogin") {
                        println!("⚠️  检测到关闭主窗口的请求，但登录窗口正在运行");
                        println!("💡 隐藏主窗口，等待登录完成后自动恢复");

                        // 阻止关闭并隐藏窗口
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
