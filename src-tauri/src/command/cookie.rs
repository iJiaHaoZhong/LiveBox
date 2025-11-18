use crate::utils::cookie_store::CookieStore;
use tauri::{AppHandle, Manager};

/// 保存用户提供的 cookie 字符串
#[tauri::command]
pub async fn save_cookies(cookie_string: String) -> Result<String, String> {
    println!("正在保存 cookies...");

    let store = CookieStore::from_cookie_string(&cookie_string, ".douyin.com");

    match CookieStore::get_default_path() {
        Ok(path) => {
            println!("Cookie 保存路径: {:?}", path);
            match store.save_to_file(&path) {
                Ok(_) => {
                    let msg = format!("成功保存 {} 个 cookies 到 {:?}", store.cookies.len(), path);
                    println!("{}", msg);
                    Ok(msg)
                }
                Err(e) => {
                    let err_msg = format!("保存 cookies 失败: {}", e);
                    eprintln!("{}", err_msg);
                    Err(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("获取保存路径失败: {}", e);
            eprintln!("{}", err_msg);
            Err(err_msg)
        }
    }
}

/// 加载已保存的 cookies
#[tauri::command]
pub async fn load_cookies() -> Result<String, String> {
    println!("正在加载 cookies...");

    match CookieStore::get_default_path() {
        Ok(path) => {
            if !path.exists() {
                return Err("Cookie 文件不存在，请先保存 cookies".to_string());
            }

            match CookieStore::load_from_file(&path) {
                Ok(store) => {
                    let cookie_str = store.to_cookie_string();
                    println!("成功加载 {} 个 cookies", store.cookies.len());
                    Ok(cookie_str)
                }
                Err(e) => {
                    let err_msg = format!("加载 cookies 失败: {}", e);
                    eprintln!("{}", err_msg);
                    Err(err_msg)
                }
            }
        }
        Err(e) => {
            let err_msg = format!("获取 cookie 路径失败: {}", e);
            eprintln!("{}", err_msg);
            Err(err_msg)
        }
    }
}

/// 清除已保存的 cookies
#[tauri::command]
pub async fn clear_cookies() -> Result<String, String> {
    println!("正在清除 cookies...");

    match CookieStore::get_default_path() {
        Ok(path) => {
            if path.exists() {
                match std::fs::remove_file(&path) {
                    Ok(_) => {
                        let msg = format!("成功清除 cookies: {:?}", path);
                        println!("{}", msg);
                        Ok(msg)
                    }
                    Err(e) => {
                        let err_msg = format!("清除 cookies 失败: {}", e);
                        eprintln!("{}", err_msg);
                        Err(err_msg)
                    }
                }
            } else {
                Ok("Cookie 文件不存在，无需清除".to_string())
            }
        }
        Err(e) => {
            let err_msg = format!("获取 cookie 路径失败: {}", e);
            eprintln!("{}", err_msg);
            Err(err_msg)
        }
    }
}

/// 打开抖音登录页面，让用户手动登录
#[tauri::command]
pub async fn open_login_page(handle: AppHandle) -> Result<String, String> {
    println!("正在打开抖音登录页面...");

    let window_label = "douyinLogin";

    // 如果窗口已存在，先关闭
    if let Some(existing_window) = handle.get_window(window_label) {
        let _ = existing_window.close();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    // 创建新窗口，注入自动提取 Cookie 的脚本
    let window = match tauri::WindowBuilder::new(
        &handle,
        window_label,
        tauri::WindowUrl::External("https://www.douyin.com/".parse().unwrap()),
    )
    .title("抖音登录 - 登录后 Cookie 会自动保存")
    .inner_size(1200.0, 800.0)
    .center()
    .initialization_script(include_str!("../inject/cookie_extractor.js"))
    .build()
    {
        Ok(win) => win,
        Err(e) => {
            let err_msg = format!("打开登录窗口失败: {}", e);
            eprintln!("{}", err_msg);
            return Err(err_msg);
        }
    };

    // 克隆 window 用于异步任务
    let window_clone = window.clone();

    // 启动一个异步任务来监听 URL 变化
    tauri::async_runtime::spawn(async move {
        use std::time::Duration;
        use tokio::time::sleep;

        let mut check_count = 0;
        let max_checks = 600; // 最多检查 10 分钟

        loop {
            check_count += 1;

            if check_count > max_checks {
                println!("⏱ Cookie 提取超时（10分钟），停止监听");
                break;
            }

            // 每秒检查一次
            sleep(Duration::from_secs(1)).await;

            // 获取当前 URL
            let current_url = window_clone.url();
            let url_str = current_url.to_string();

            // 检查是否有 Cookie 数据返回
            if url_str.contains("#__COOKIES__=") {
                println!("🍪 检测到 Cookie 数据！");

                if let Some(hash_start) = url_str.find("#__COOKIES__=") {
                    let cookie_data = &url_str[hash_start + 13..]; // "#__COOKIES__=" 长度为 13

                    match urlencoding::decode(cookie_data) {
                        Ok(decoded_cookies) => {
                            println!("✅ Cookie 数据解码成功");

                            // 保存 Cookie 到文件
                            let cookie_string = decoded_cookies.to_string();
                            let store = CookieStore::from_cookie_string(&cookie_string, ".douyin.com");

                            match CookieStore::get_default_path() {
                                Ok(path) => {
                                    match store.save_to_file(&path) {
                                        Ok(_) => {
                                            println!("💾 成功保存 {} 个 cookies 到 {:?}", store.cookies.len(), path);

                                            // 等待 2 秒后关闭窗口（让用户看到成功提示）
                                            sleep(Duration::from_secs(2)).await;
                                            let _ = window_clone.close();
                                            println!("✅ 登录窗口已关闭");
                                        }
                                        Err(e) => {
                                            eprintln!("❌ 保存 cookies 失败: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ 获取保存路径失败: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Cookie 数据解码失败: {}", e);
                        }
                    }
                }

                // 找到 Cookie 后停止循环
                break;
            }
        }
    });

    Ok("登录窗口已打开，请在浏览器中登录抖音".to_string())
}
