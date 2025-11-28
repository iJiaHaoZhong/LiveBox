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
            command::cookie::open_login_page,
            command::taobao::start_taobao_crawler,
            command::taobao::stop_taobao_crawler,
            command::taobao::check_taobao_crawler_status
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
        .on_page_load(|window, _payload| {
            // 确保登录窗口创建后打印日志
            if window.label() == "douyinLogin" {
                println!("📱 登录窗口页面已加载: {}", window.label());
            }
        })
        .setup(|app| {
            // 创建隐藏的守护窗口，防止应用退出
            let _daemon = tauri::WindowBuilder::new(
                app,
                "daemon",
                tauri::WindowUrl::App("index.html".into())
            )
            .title("LiveBox Daemon")
            .inner_size(1.0, 1.0)
            .visible(false)
            .skip_taskbar(true)
            .build()?;

            println!("🛡️ 守护窗口已创建，应用不会自动退出");

            // 启动时检查 Cookie 文件状态
            use utils::cookie_store::CookieStore;
            if let Ok(cookie_path) = CookieStore::get_default_path() {
                println!("📁 Cookie 文件位置: {:?}", cookie_path);
                if cookie_path.exists() {
                    match CookieStore::load_from_file(&cookie_path) {
                        Ok(store) => {
                            println!("✅ 发现已保存的 Cookie 文件，包含 {} 个 Cookie", store.cookies.len());
                        }
                        Err(e) => {
                            println!("⚠️  Cookie 文件存在但读取失败: {}", e);
                        }
                    }
                } else {
                    println!("ℹ️  Cookie 文件尚未创建，首次登录后将自动保存");
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // 全局事件处理 - 防止在登录期间退出应用
            match event {
                tauri::RunEvent::ExitRequested { api, .. } => {
                    // 检查是否有登录窗口在运行
                    if let Some(_login_window) = app_handle.get_window("douyinLogin") {
                        println!("🛑 检测到退出请求，但登录窗口正在运行");
                        println!("💡 阻止应用退出，等待登录完成");
                        api.prevent_exit();
                    } else {
                        // 允许正常退出，但先关闭守护窗口
                        if let Some(daemon) = app_handle.get_window("daemon") {
                            let _ = daemon.close();
                        }
                    }
                }
                _ => {}
            }
        });
}
