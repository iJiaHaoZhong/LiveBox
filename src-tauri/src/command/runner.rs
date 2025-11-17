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
        let client = Client::builder().cookie_store(true).build().unwrap();
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
        home_headers.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse()?);
        home_headers.insert("accept-language", "zh-CN,zh;q=0.9,en;q=0.8".parse()?);
        home_headers.insert("cache-control", "max-age=0".parse()?);
        home_headers.insert("sec-ch-ua", "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"".parse()?);
        home_headers.insert("sec-ch-ua-mobile", "?0".parse()?);
        home_headers.insert("sec-ch-ua-platform", "\"Windows\"".parse()?);
        home_headers.insert("sec-fetch-dest", "document".parse()?);
        home_headers.insert("sec-fetch-mode", "navigate".parse()?);
        home_headers.insert("sec-fetch-site", "none".parse()?);
        home_headers.insert("sec-fetch-user", "?1".parse()?);
        home_headers.insert("upgrade-insecure-requests", "1".parse()?);
        home_headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".parse()?);

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
            if cookie_path.exists() {
                match CookieStore::load_from_file(&cookie_path) {
                    Ok(store) => {
                        println!("✓ 成功加载 {} 个已保存的用户 Cookie", store.cookies.len());
                        Some(store.to_cookie_string())
                    }
                    Err(e) => {
                        println!("⚠ 加载保存的 Cookie 失败: {}", e);
                        None
                    }
                }
            } else {
                println!("ℹ 未找到保存的 Cookie 文件，使用默认请求");
                None
            }
        } else {
            None
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse()?);
        headers.insert("accept-language", "zh-CN,zh;q=0.9,en;q=0.8".parse()?);
        headers.insert("cache-control", "max-age=0".parse()?);
        headers.insert("priority", "u=0, i".parse()?);
        headers.insert("referer", "https://www.douyin.com/".parse()?);
        headers.insert("sec-ch-ua", "\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"".parse()?);
        headers.insert("sec-ch-ua-mobile", "?0".parse()?);
        headers.insert("sec-ch-ua-platform", "\"Windows\"".parse()?);
        headers.insert("sec-fetch-dest", "document".parse()?);
        headers.insert("sec-fetch-mode", "navigate".parse()?);
        headers.insert("sec-fetch-site", "same-origin".parse()?);
        headers.insert("sec-fetch-user", "?1".parse()?);
        headers.insert("upgrade-insecure-requests", "1".parse()?);
        headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36".parse()?);

        // 如果有保存的 Cookie，添加到请求头
        if let Some(cookie_str) = saved_cookies {
            headers.insert("cookie", cookie_str.parse()?);
            println!("✓ 已将保存的 Cookie 添加到请求头");
        }

        let request = self.request.get(self.room_url.clone()).headers(headers);
        let response = request.send().await?;
        // 先使用cookie，再使用text
        let cookies = response.cookies();
        let mut ttwid = String::new();
        for c in cookies {
            println!("cookies: {:?} value:{:?}", c.name(), c.value());
            if c.name() == "ttwid" {
                ttwid = c.value().to_string();
            }
        }
        // 获取cookie里面的ttwid
        let body = response.text().await?;

        // 检测是否为 Access Denied 错误
        if body.contains("Access Denied") || body.contains("X-TT-System-Error") {
            println!("❌ 检测到 Access Denied 错误，需要登录");
            println!("💡 提示: 请调用 open_login_page 命令登录抖音账号");
            return Err(crate::command::model::ERROR_ACCESS_DENIED.into());
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
                    return Err("无法提取房间信息 (group 1 不存在)".into());
                }
            },
            None => {
                return Err("无法匹配房间信息，可能直播间地址无效或页面结构已变化".into());
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
