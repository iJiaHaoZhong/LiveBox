use crate::command::model::LiveInfo;
use tauri::{AppHandle, Manager};

// 自定义函数
#[tauri::command]
pub async fn greet_you(name: &str) -> Result<String, String> {
    println!("调用了greet_you");
    Ok(format!("Hello, {}! You've been greeted from Rust!", name))
}

#[tauri::command]
pub async fn get_live_html(url: &str, handle: AppHandle) -> Result<LiveInfo, String> {
    println!("🎯 [get_live_html] 开始执行，URL: {}", url);
    println!("🌐 [get_live_html] 使用浏览器窗口提取数据（方案1）");
    println!("💡 [get_live_html] 不使用后端 HTTP 请求，直接在浏览器中提取数据");

    // ========== 步骤1: 先发送 HEAD 请求获取 ttwid Cookie ==========
    println!("🍪 [get_live_html] 步骤1: 获取 ttwid Cookie...");
    let mut extracted_ttwid = String::new();

    match reqwest::Client::builder()
        .cookie_store(true)
        .build()
    {
        Ok(client) => {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());

            match client.head(url).headers(headers).send().await {
                Ok(response) => {
                    println!("📊 [get_live_html] HEAD 请求响应状态: {}", response.status());

                    // 从响应的 Set-Cookie 中提取 ttwid
                    let cookies = response.cookies();
                    for cookie in cookies {
                        if cookie.name() == "ttwid" {
                            extracted_ttwid = cookie.value().to_string();
                            println!("✅ [get_live_html] 成功提取 ttwid: {}...", &extracted_ttwid[..20.min(extracted_ttwid.len())]);
                            break;
                        }
                    }

                    if extracted_ttwid.is_empty() {
                        println!("⚠️  [get_live_html] HEAD 请求未返回 ttwid，尝试 GET 请求...");

                        // 如果 HEAD 没有返回 ttwid，尝试 GET
                        match client.get(url).headers(headers).send().await {
                            Ok(get_response) => {
                                let get_cookies = get_response.cookies();
                                for cookie in get_cookies {
                                    if cookie.name() == "ttwid" {
                                        extracted_ttwid = cookie.value().to_string();
                                        println!("✅ [get_live_html] GET 请求成功提取 ttwid: {}...", &extracted_ttwid[..20.min(extracted_ttwid.len())]);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                println!("⚠️  [get_live_html] GET 请求失败: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  [get_live_html] HEAD 请求失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ [get_live_html] 无法创建 HTTP 客户端: {}", e);
        }
    }

    if extracted_ttwid.is_empty() {
        println!("⚠️  [get_live_html] 未能获取 ttwid，WebSocket 连接可能会失败");
    }

    // ========== 步骤2: 打开浏览器窗口提取数据 ==========
    let window_label = "douyinData";

    // 如果窗口已存在，先关闭
    if let Some(existing_window) = handle.get_window(window_label) {
        let _ = existing_window.close();
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // 创建窗口，注入数据提取脚本
    println!("🪟 [get_live_html] 步骤2: 打开浏览器窗口...");
    match tauri::WindowBuilder::new(
        &handle,
        window_label,
        tauri::WindowUrl::External(url.parse().unwrap()),
    )
    .title("正在获取直播间数据...")
    .inner_size(1200.0, 800.0)
    .center()
    .initialization_script(include_str!("../inject/data_extractor.js"))
    .build()
    {
        Ok(window) => {
            println!("✅ [get_live_html] 窗口已打开");
            println!("⏳ [get_live_html] 等待数据提取...");

            let mut attempts = 0;
            let max_attempts = 120; // 60 秒（每次检查间隔 500ms）

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // 检查窗口是否还存在
                if handle.get_window(window_label).is_none() {
                    println!("⚠️  [get_live_html] 窗口已关闭");
                    return Err("窗口被用户关闭".into());
                }

                let current_url = window.url();
                let url_str = current_url.to_string();

                // 检查是否有数据返回
                if url_str.contains("#__LIVE_DATA__=") {
                    if let Some(hash_start) = url_str.find("#__LIVE_DATA__=") {
                        let data_str = &url_str[hash_start + 15..];

                        match urlencoding::decode(data_str) {
                            Ok(decoded_data) => {
                                println!("📦 [get_live_html] 接收到数据！");

                                // 解析 JSON 数据
                                match serde_json::from_str::<serde_json::Value>(&decoded_data) {
                                    Ok(data) => {
                                        println!("✅ [get_live_html] 数据解析成功！");

                                        // 提取字段并映射到 LiveInfo 结构
                                        let title = data.get("title")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        let unique_id = data.get("user_unique_id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        // room_info 存储完整的数据 JSON
                                        let room_info = data.get("room_store")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| decoded_data.to_string());

                                        // ttwid 优先使用从 HTTP 请求提取的，如果没有则尝试从 JavaScript 提取
                                        let js_ttwid = data.get("ttwid")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        let ttwid = if !extracted_ttwid.is_empty() {
                                            extracted_ttwid.clone()
                                        } else if !js_ttwid.is_empty() {
                                            js_ttwid
                                        } else {
                                            String::new()
                                        };

                                        println!("📝 标题: {}", title);
                                        println!("👤 主播ID: {}", unique_id);
                                        println!("🍪 ttwid: {}", if ttwid.is_empty() { "(未提取)" } else { "已提取" });
                                        println!("📊 room_info 长度: {} 字符", room_info.len());

                                        // 验证数据完整性：必须有标题 AND (主播ID 或 room_info)
                                        let has_valid_data = !title.is_empty() && (!unique_id.is_empty() || room_info.len() > 100);

                                        if !has_valid_data {
                                            if title.is_empty() {
                                                println!("⚠️  [get_live_html] 数据不完整：标题为空，继续等待...");
                                            } else if unique_id.is_empty() && room_info.len() <= 100 {
                                                println!("⚠️  [get_live_html] 数据不完整：缺少主播ID和完整数据，继续等待...");
                                                println!("💡 提示：请在浏览器控制台查看提取日志，了解提取情况");
                                            }
                                            // 不关闭窗口，继续等待
                                        } else {
                                            println!("✅ [get_live_html] 数据验证通过，关闭窗口");
                                            // 关闭窗口
                                            let _ = window.close();

                                            // 返回数据
                                            return Ok(LiveInfo {
                                                room_info,
                                                ttwid,
                                                unique_id,
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        println!("❌ [get_live_html] JSON 解析失败: {}", e);
                                        let _ = window.close();
                                        return Err(format!("数据解析失败: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ [get_live_html] URL 解码失败: {}", e);
                            }
                        }
                    }
                }

                // 检查是否有错误返回
                if url_str.contains("#__LIVE_ERROR__=") {
                    if let Some(hash_start) = url_str.find("#__LIVE_ERROR__=") {
                        let error_str = &url_str[hash_start + 16..];

                        match urlencoding::decode(error_str) {
                            Ok(decoded_error) => {
                                match serde_json::from_str::<serde_json::Value>(&decoded_error) {
                                    Ok(error_data) => {
                                        let error_type = error_data.get("error")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown");

                                        let error_message = error_data.get("message")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("未知错误");

                                        println!("❌ [get_live_html] 提取失败: {} - {}", error_type, error_message);
                                        let _ = window.close();
                                        return Err(format!("数据提取失败: {}", error_message));
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }

                attempts += 1;
                if attempts >= max_attempts {
                    println!("⏱ [get_live_html] 等待超时（60秒）");
                    let _ = window.close();
                    return Err("数据提取超时".into());
                }

                if attempts % 10 == 0 {
                    println!("⏳ [get_live_html] 等待中... ({} 秒)", attempts / 2);
                }
            }
        }
        Err(e) => {
            println!("❌ [get_live_html] 无法打开窗口: {}", e);
            Err(format!("无法打开窗口: {}", e))
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
    println!("Opening docs in external window: {}, {}", app_url, platform);

    if !resize {
        let _window = tauri::WindowBuilder::new(
            &handle,
            window_label,
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
