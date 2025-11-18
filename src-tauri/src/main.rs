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
        .on_window_event(|event| {
            // 防止关闭主窗口时退出应用（如果还有其他窗口在运行）
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                let window = event.window();

                // 如果这不是登录窗口，检查是否有登录窗口正在运行
                if window.label() != "douyinLogin" {
                    if let Some(_login_window) = window.app_handle().get_window("douyinLogin") {
                        println!("⚠️  检测到关闭主窗口的请求，但登录窗口正在运行");
                        println!("💡 请等待登录完成，或关闭登录窗口后再关闭主窗口");
                        api.prevent_close();
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
