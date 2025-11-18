use crate::command::model::LiveInfo;
use crate::utils::cookie_store::CookieStore;
use regex::Regex;
use reqwest::Client;

// 定义抖音请求结构体
pub struct DouYinReq {
    request: Client,
    room_url: String,
    room_info: String,
}

// 为抖音请求的结构体添加方法
impl DouYinReq {
    pub fn new(url: &str) -> Self {
        // 配置 HTTP 客户端以模拟浏览器行为
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        DouYinReq {
            request: client,
            room_url: String::from(url),
            room_info: String::from(""),
        }
    }

    pub async fn get_room_info(&mut self) -> Result<LiveInfo, Box<dyn std::error::Error>> {
        println!("获取直播间的room_info: {}", self.room_url);

        // 第一步：先访问 douyin.com 主页，获取必要的 Cookie（避免 Access Denied）
        println!("步骤1: 访问 douyin.com 获取初始 Cookie...");
        let mut home_headers = reqwest::header::HeaderMap::new();
        home_headers.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse()?);
        // 移除 accept-encoding 以获取未压缩响应（reqwest 需要额外 features 才能自动解压）
        home_headers.insert("accept-language", "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6".parse()?);
        home_headers.insert("cache-control", "max-age=0".parse()?);
        home_headers.insert("dnt", "1".parse()?);
        home_headers.insert("sec-ch-ua", "\"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".parse()?);
        home_headers.insert("sec-ch-ua-mobile", "?0".parse()?);
        home_headers.insert("sec-ch-ua-platform", "\"Windows\"".parse()?);
        home_headers.insert("sec-fetch-dest", "document".parse()?);
        home_headers.insert("sec-fetch-mode", "navigate".parse()?);
        home_headers.insert("sec-fetch-site", "none".parse()?);
        home_headers.insert("sec-fetch-user", "?1".parse()?);
        home_headers.insert("upgrade-insecure-requests", "1".parse()?);
        home_headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse()?);

        // 添加人类行为模拟：延迟 1 秒后再访问（模拟用户浏览行为）
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let home_response = self.request.get("https://www.douyin.com/").headers(home_headers).send().await?;
        let home_cookies = home_response.cookies();
        let mut collected_cookies = Vec::new();
        for c in home_cookies {
            collected_cookies.push(format!("{}={}", c.name(), c.value()));
            println!("  获取到 Cookie: {}", c.name());
        }

        // 第二步：使用获取的 Cookie 访问直播间页面
        println!("步骤2: 使用 Cookie 访问直播间...");

        // 尝试加载用户保存的 Cookie
        let saved_cookies = if let Ok(cookie_path) = CookieStore::get_default_path() {
            println!("📁 Cookie 文件路径: {:?}", cookie_path);
            println!("📁 文件是否存在: {}", cookie_path.exists());

            if cookie_path.exists() {
                match CookieStore::load_from_file(&cookie_path) {
                    Ok(store) => {
                        println!("✅ 成功加载 {} 个已保存的用户 Cookie", store.cookies.len());
                        let cookie_str = store.to_cookie_string();
                        println!("🍪 Cookie 内容预览: {}...", &cookie_str.chars().take(100).collect::<String>());
                        Some(cookie_str)
                    }
                    Err(e) => {
                        println!("⚠️ 加载保存的 Cookie 失败: {}", e);
                        None
                    }
                }
            } else {
                println!("ℹ️ 未找到保存的 Cookie 文件: {:?}", cookie_path);
                println!("💡 如果您已登录过，请检查文件是否被删除");
                None
            }
        } else {
            println!("❌ 无法获取 Cookie 文件路径");
            None
        };

        let mut headers = reqwest::header::HeaderMap::new();
        // 严格按照浏览器请求头的顺序和格式
        headers.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse()?);
        // 移除 accept-encoding 以获取未压缩响应
        headers.insert("accept-language", "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6".parse()?);
        headers.insert("cache-control", "max-age=0".parse()?);
        headers.insert("dnt", "1".parse()?); // Do Not Track
        headers.insert("priority", "u=0, i".parse()?);
        headers.insert("referer", "https://www.douyin.com/".parse()?);
        headers.insert("sec-ch-ua", "\"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\"".parse()?);
        headers.insert("sec-ch-ua-mobile", "?0".parse()?);
        headers.insert("sec-ch-ua-platform", "\"Windows\"".parse()?);
        headers.insert("sec-fetch-dest", "document".parse()?);
        headers.insert("sec-fetch-mode", "navigate".parse()?);
        headers.insert("sec-fetch-site", "same-origin".parse()?);
        headers.insert("sec-fetch-user", "?1".parse()?);
        headers.insert("upgrade-insecure-requests", "1".parse()?);
        headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse()?);

        // 如果有保存的 Cookie，添加到请求头
        let using_saved_cookies = saved_cookies.is_some();
        if let Some(ref cookie_str) = saved_cookies {
            headers.insert("cookie", cookie_str.parse()?);
            println!("✓ 已将保存的 Cookie 添加到请求头");
            println!("📋 Cookie 详情（前200字符）: {}...", &cookie_str.chars().take(200).collect::<String>());
        } else {
            println!("ℹ️  未使用保存的 Cookie，仅使用从主页获取的临时 Cookie");
        }

        println!("🌐 开始发送请求到直播间页面...");
        let request = self.request.get(self.room_url.clone()).headers(headers);
        let response = request.send().await?;

        // 记录响应状态
        let status = response.status();
        println!("📊 响应状态码: {}", status);

        // 先使用cookie，再使用text
        let cookies = response.cookies();
        let mut ttwid = String::new();
        println!("🍪 从响应中获取的 Cookie:");
        for c in cookies {
            println!("   - {}: {} (domain: {:?}, path: {:?})",
                c.name(),
                if c.value().len() > 50 { format!("{}...", &c.value()[..50]) } else { c.value().to_string() },
                c.domain(),
                c.path()
            );
            if c.name() == "ttwid" {
                ttwid = c.value().to_string();
            }
        }

        // 获取cookie里面的ttwid
        println!("📄 开始读取响应内容...");
        let body = response.text().await?;
        println!("📏 响应内容长度: {} 字符", body.len());

        // 显示响应内容的开头和结尾（用于调试）
        if body.len() > 0 {
            let preview_start = body.chars().take(500).collect::<String>();
            let preview_end = if body.len() > 500 {
                body.chars().skip(body.len().saturating_sub(300)).collect::<String>()
            } else {
                String::new()
            };
            println!("📄 响应内容预览（前500字符）:");
            println!("{}", preview_start);
            if !preview_end.is_empty() {
                println!("📄 响应内容预览（最后300字符）:");
                println!("{}", preview_end);
            }
        }

        // 检测是否需要登录或验证码
        let mut deny_reason = None;
        let mut is_captcha = false;

        // 优先检测验证码页面（这些需要用户交互）
        if body.contains("验证码中间页") || body.contains("middle_page_loading") {
            deny_reason = Some("包含 '验证码中间页' 或 'middle_page_loading' - 需要用户完成验证码");
            is_captcha = true;
        } else if body.contains("captcha") {
            deny_reason = Some("包含 'captcha' 验证码标识 - 需要用户完成验证码");
            is_captcha = true;
        } else if body.contains("Access Denied") {
            deny_reason = Some("包含 'Access Denied' 文字 - 访问被拒绝");
        } else if body.contains("X-TT-System-Error") {
            deny_reason = Some("包含 'X-TT-System-Error' 系统错误标识");
        }

        if let Some(reason) = deny_reason {
            println!("\n❌ ========== 访问被拒绝 ==========");
            println!("❌ 检测到需要{}验证", if is_captcha { "验证码" } else { "登录或" });
            println!("📍 拒绝原因: {}", reason);
            println!("🍪 是否使用了保存的 Cookie: {}", if using_saved_cookies { "是" } else { "否" });
            if using_saved_cookies {
                if let Some(ref cookie_str) = saved_cookies {
                    println!("📋 使用的 Cookie 数量: {} 个", cookie_str.split(';').count());
                    println!("📋 Cookie 示例:");
                    for (i, cookie) in cookie_str.split(';').take(5).enumerate() {
                        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let value_preview = if parts[1].len() > 30 {
                                format!("{}...", &parts[1][..30])
                            } else {
                                parts[1].to_string()
                            };
                            println!("   {}. {} = {}", i + 1, parts[0], value_preview);
                        }
                    }
                }
            }
            println!("🌐 请求的 URL: {}", self.room_url);

            if is_captcha {
                println!("💡 提示: 需要打开浏览器窗口让用户完成验证码验证");
                println!("======================================\n");
                return Err(crate::command::model::ERROR_CAPTCHA_REQUIRED.into());
            } else {
                println!("💡 提示: 后端将根据 Cookie 文件是否存在决定是否打开登录窗口");
                println!("======================================\n");
                return Err(crate::command::model::ERROR_ACCESS_DENIED.into());
            }
        }

        // println!("获取的直播间HTML内容是：{}", body);
        // 判断是不是已经停播了，是的话仅获取主播头像
        // 使用正则表达式匹配直播间信息
        let re;
        let mut unique_id = "";
        if body.contains(r#"status\":4"#) {
            println!("主播已停播了");
            // 使用正则表达式匹配直播间信息
            re = Regex::new(r#"anchor\\":(.*?),\\"open_id_str"#).unwrap();
        } else {
            // 使用正则表达式匹配直播间信息
            re = Regex::new(r#"roomInfo\\":\{\\"room\\":(.*?),\\"toolbar_data"#).unwrap();

            // 尝试多种正则模式来匹配 user_unique_id
            let patterns = vec![
                r#"user_unique_id\\":\\"(.*?)\\"}"#,           // 原始模式
                r#"user_unique_id":"([^"]+)"#,                 // 不带转义的模式
                r#"user_unique_id\\":\\"([^\\]+)\\"#,         // 更宽松的模式
                r#""user_unique_id":"([^"]+)""#,               // JSON 格式
            ];

            // 依次尝试每种模式
            let mut matched = false;
            for pattern in patterns.iter() {
                if let Ok(unique_re) = Regex::new(pattern) {
                    if let Some(captures) = unique_re.captures(&body) {
                        if let Some(m) = captures.get(1) {
                            unique_id = m.as_str();
                            println!("✓ 成功提取 unique_id: {} (使用模式: {})", unique_id, pattern);
                            matched = true;
                            break;
                        }
                    }
                }
            }

            if !matched {
                println!("⚠ 警告: 所有正则模式都无法匹配 user_unique_id");
                println!("  这可能是因为:");
                println!("  1. 页面结构已变化");
                println!("  2. 需要登录才能访问");
                println!("  3. 直播间不存在或已关闭");

                // 输出部分 body 内容用于调试（仅前 500 字符，避免输出过多）
                let preview_len = 500.min(body.len());
                println!("  HTML 预览 (前 {} 字符):", preview_len);
                println!("  {}", &body[..preview_len]);
            }
        }

        // 安全地获取房间信息
        let main_info = match re.captures(&body) {
            Some(captures) => match captures.get(1) {
                Some(matched) => matched.as_str(),
                None => {
                    println!("❌ 无法提取房间信息，可能需要登录");
                    return Err(crate::command::model::ERROR_ACCESS_DENIED.into());
                }
            },
            None => {
                println!("❌ 无法匹配房间信息，可能需要登录或页面结构已变化");
                println!("💡 提示: 后端将自动打开登录窗口");
                return Err(crate::command::model::ERROR_ACCESS_DENIED.into());
            }
        };
        // 替换里面的双引号,方便json解析
        let room_info = String::from(main_info) + "}";
        self.room_info = room_info.replace(r#"\""#, r#"""#);
        // println!("直播间信息是：{}", self.room_info);
        Ok(LiveInfo {
            room_info: self.room_info.clone(),
            ttwid,
            unique_id: String::from(unique_id),
        })
    }

    // pub async fn get_rank_info(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut headers = reqwest::header::HeaderMap::new();
    //     headers.insert("accept", "application/json, text/plain, */*".parse()?);
    //     headers.insert("accept-language", "zh-CN,zh;q=0.9,en;q=0.8".parse()?);
    //     headers.insert("cache-control", "no-cache".parse()?);
    //     headers.insert("cookie", "has_avx2=null; device_web_cpu_core=8; device_web_memory_size=8; live_use_vvc=%22false%22; xgplayer_user_id=32142398740; csrf_session_id=b154f2eb3608feb421dd6c9fe24bc2d4; odin_tt=a5b308e92c2f826f447b22425cb49c1faa5a13b64c07a6f7309186819371d4c74fe5dcf480e52fe2931fba91397a83a31c94e2df31a3735b839683d58bf010781b5c5f61c231ab52f5ecfbc03f80ff23; passport_csrf_token=6bc63b63e5fe245d323c824928bc812e; passport_csrf_token_default=6bc63b63e5fe245d323c824928bc812e; bd_ticket_guard_client_web_domain=2; webcast_local_quality=sd; SEARCH_RESULT_LIST_TYPE=%22single%22; ttwid=1%7CUOwlzl-VvV0COewDTk3CsEdp4EMg8CUFA-ICTdsrLQw%7C1712887757%7Cb40c2475ea6f287e8da8722ef9dfcb4b1b9d35e05158a6fc6dbc3282a4caf15a; __ac_nonce=0662a095500344b59f1a0; __ac_signature=_02B4Z6wo00f01OHJxKwAAIDDszbktR5C2nTh6cAAAF5e7JV0RQje.O9NY-t5t6vN9NKbPcnfXMkFfQLkfKLc17gPyPteEs6w5xUu7in-FxDZfmcOuSUKGOIUEeUxSuh0vbz9E.lVYSPp2boo0f; webcast_leading_last_show_time=1714030934972; webcast_leading_total_show_times=4; bd_ticket_guard_client_data=eyJiZC10aWNrZXQtZ3VhcmQtdmVyc2lvbiI6MiwiYmQtdGlja2V0LWd1YXJkLWl0ZXJhdGlvbi12ZXJzaW9uIjoxLCJiZC10aWNrZXQtZ3VhcmQtcmVlLXB1YmxpYy1rZXkiOiJCRExvdFozTlZJU3ZpQjZ3YzREeHdSdTYwaVY1eTIwUzM1UytLTllwTUs0Tmxoc3M3Z1ZjdFpYWmhiQ0ZWTzYrNEVsSGd0U25GM1BERWc4UFgvZFFodVE9IiwiYmQtdGlja2V0LWd1YXJkLXdlYi12ZXJzaW9uIjoxfQ%3D%3D; download_guide=%223%2F20240425%2F0%22; pwa2=%220%7C0%7C3%7C0%22; FORCE_LOGIN=%7B%22videoConsumedRemainSeconds%22%3A180%2C%22isForcePopClose%22%3A1%7D; home_can_add_dy_2_desktop=%221%22; __live_version__=%221.1.1.9809%22; xg_device_score=7.541386294591826; live_can_add_dy_2_desktop=%220%22; IsDouyinActive=true; msToken=LrwiNPyulLPWEKS-5jj4OvncuOKQA8y4qFfo1j-JN2Yw3-eg_j-DrE_CKTQmOz44dwG26uOxevFyITDrkPwx82M4k4XvQ8zgm3MjnQDDmtZ89Yikpkve-kRMQSuo; msToken=Qj3DmdHUf10MnlDFyLJeQaF1tLaXa93UwyL2V84tV9u8B0JAp1RuVZC41Lzw066HS7G2rqUkiQB-7DCWhkiEmQlD3KyucfKG5qPdUY3jEo39oRyafq4M2cpXm8Mv; ttwid=1%7CngabJA52sDUnYMxFKTFQmYEe2_RYNkefWVWEfuA53Mo%7C1713104743%7C34512c898d125865794d949a2477dda7493530c850da7c59a19c32a46642876c".parse()?);
    //     headers.insert("pragma", "no-cache".parse()?);
    //     headers.insert("priority", "u=1, i".parse()?);
    //     headers.insert(
    //         "sec-ch-ua",
    //         "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\""
    //             .parse()?,
    //     );
    //     headers.insert("sec-ch-ua-mobile", "?0".parse()?);
    //     headers.insert("sec-ch-ua-platform", "\"macOS\"".parse()?);
    //     headers.insert("sec-fetch-dest", "empty".parse()?);
    //     headers.insert("sec-fetch-mode", "cors".parse()?);
    //     headers.insert("sec-fetch-site", "same-origin".parse()?);
    //     headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".parse()?);
    //     let request = self.request.get(format!("https://live.douyin.com/webcast/ranklist/audience/?aid=6383&app_name=douyin_web&live_id=1&device_platform=web&language=zh-CN&cookie_enabled=true&screen_width=2560&screen_height=1440&browser_language=zh-CN&browser_platform=Win32&browser_name=Chrome&browser_version=117.0.0.0&webcast_sdk_version=2450&room_info={}&rank_type=30", self.room_info)).headers(headers);
    //     let response = request.send().await?;
    //     let json_value = response.text().await?;
    //     println!("rank json value:{json_value:?}");
    //     Ok(())
    // }
}
