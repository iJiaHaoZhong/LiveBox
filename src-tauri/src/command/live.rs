use crate::command::model::{LiveInfo, ERROR_ACCESS_DENIED};
use crate::command::runner::DouYinReq;
use tauri::{AppHandle, Manager};

// 自定义函数
#[tauri::command]
pub async fn greet_you(name: &str) -> Result<String, String> {
    println!("调用了greet_you");
    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

#[tauri::command]
pub async fn get_live_html(url: &str, handle: AppHandle) -> Result<LiveInfo, String> {
    let mut live_req = DouYinReq::new(url);

    // 第一次尝试获取直播间信息
    let result = live_req.get_room_info().await;

    // 立即将 Result 转换为 Result<LiveInfo, String>，避免 Send 问题
    let result_string: Result<LiveInfo, String> = result.map_err(|e| e.to_string());

    match result_string {
        Ok(info) => Ok(info),
        Err(error_msg) => {
            // 检查是否为 Access Denied 错误
            if error_msg == ERROR_ACCESS_DENIED {
                println!("🔐 检测到需要登录，自动打开登录窗口...");

                // 自动打开登录窗口
                let window_label = "douyinLogin";

                // 如果窗口已存在，先关闭
                if let Some(existing_window) = handle.get_window(window_label) {
                    let _ = existing_window.close();
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }

                // 创建新窗口
                match tauri::WindowBuilder::new(
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
                    Ok(window) => {
                        println!("✅ 登录窗口已打开");
                        println!("⏳ 等待用户登录...");
                        println!("💡 提示: 请在打开的窗口中登录，登录成功后窗口会自动关闭");

                        // 定期检查窗口标题以获取 Cookie（最多等待 120 秒）
                        let mut attempts = 0;
                        let max_attempts = 240; // 120秒 (每次检查间隔 500ms)
                        let mut cookie_string: Option<String> = None;

                        loop {
                            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                            // 检查窗口是否还存在
                            if handle.get_window(window_label).is_none() {
                                println!("✅ 登录窗口已关闭");
                                break;
                            }

                            // 尝试从窗口标题读取 Cookie
                            if cookie_string.is_none() {
                                if let Ok(title) = window.title() {
                                    if title.starts_with("__COOKIES_READY__|") {
                                        // 提取 Cookie 字符串
                                        let cookies = title.trim_start_matches("__COOKIES_READY__|");
                                        cookie_string = Some(cookies.to_string());

                                        println!("🍪 检测到 Cookie！");
                                        println!("📝 Cookie 长度: {} 字符", cookies.len());

                                        // 保存 Cookie
                                        match crate::command::cookie::save_cookies(cookies.to_string()).await {
                                            Ok(msg) => {
                                                println!("✅ {}", msg);
                                            }
                                            Err(err) => {
                                                eprintln!("❌ Cookie 保存失败: {}", err);
                                            }
                                        }

                                        // 关闭窗口
                                        let _ = window.close();
                                        println!("🔒 登录窗口已关闭");
                                        break;
                                    }
                                }
                            }

                            attempts += 1;
                            if attempts >= max_attempts {
                                println!("⏱ 等待超时（120秒），未检测到登录");
                                let _ = window.close();
                                return Err("等待登录超时，请重试".into());
                            }

                            // 每 10 秒提示一次
                            if attempts % 20 == 0 {
                                let seconds = attempts / 2;
                                println!("⏳ 已等待 {} 秒，请尽快完成登录...", seconds);
                            }
                        }

                        // 检查是否成功获取到 Cookie
                        if cookie_string.is_none() {
                            println!("⚠️  窗口已关闭，但未检测到 Cookie");
                            return Err("未检测到登录 Cookie，请重试".into());
                        }

                        // 等待额外 1 秒确保 Cookie 已保存到文件
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        println!("🔄 重试获取直播间信息...");

                        // 重新创建请求（使用新的 Cookie）
                        let mut retry_req = DouYinReq::new(url);
                        let retry_result = retry_req.get_room_info().await;

                        match retry_result {
                            Ok(info) => {
                                println!("✅ 登录成功，成功获取直播间信息！");
                                Ok(info)
                            }
                            Err(retry_error) => {
                                let retry_msg = retry_error.to_string();
                                if retry_msg == ERROR_ACCESS_DENIED {
                                    Err("登录可能未完成或失败，请重试".into())
                                } else {
                                    Err(format!("重试失败: {}", retry_msg))
                                }
                            }
                        }
                    }
                    Err(window_err) => {
                        eprintln!("❌ 打开登录窗口失败: {}", window_err);
                        Err(format!("无法打开登录窗口: {}", window_err))
                    }
                }
            } else {
                // 其他错误直接返回
                Err(error_msg)
            }
        }
    }
}

#[tauri::command]
pub async fn open_window(
    handle: AppHandle,
    app_url: String,
    app_name: String,
    platform: String,
    user_agent: String,
    resize: bool,
    width: f64,
    height: f64,
    _js_content: String,
) {
    let window_label = "previewWeb";
    // if let Some(existing_window) = handle.get_window(window_label) {
    //     if resize {
    //         let new_size = LogicalSize::new(width, height);
    //         match existing_window.set_size(new_size) {
    //             Ok(_) => println!("Window resized to {}x{}", width, height),
    //             Err(e) => eprintln!("Failed to resize window: {}", e),
    //         }
    //     } else {
    //         existing_window.close().unwrap();
    //         println!("Existing window closed.");
    //         let start = Instant::now();
    //         while handle.get_window(window_label).is_some() {
    //             if start.elapsed().as_secs() > 2 {
    //                 println!("Window close took too long. Aborting.");
    //                 return;
    //             }
    //             std::thread::yield_now();
    //         }
    //     }
    // }
    println!("Opening docs in external window: {}, {}", app_url, platform);
    // println!("js_content: {}", js_content);
    // let resource_path = handle
    //     .path_resolver()
    //     .resolve_resource("data/custom.js")
    //     .expect("failed to resolve resource");
    // let mut custom_js = std::fs::File::open(&resource_path).unwrap();
    // let mut contents = String::new();
    // custom_js.read_to_string(&mut contents).unwrap();
    // contents += js_content.as_str();
    // println!("js file contents: {}", contents);
    if !resize {
        let _window = tauri::WindowBuilder::new(
            &handle,
            window_label, /* the unique window label */
            tauri::WindowUrl::External(app_url.parse().unwrap()),
        )
        .title(app_name.clone())
        .inner_size(width, height)
        .user_agent(user_agent.as_str())
        .initialization_script(include_str!("../inject/websocket.js"))
        .center()
        .build()
        .unwrap();
    }
}
