// 自定义返回的消息
#[derive(serde::Serialize)]
pub struct LiveInfo {
    pub room_info: String,
    pub ttwid: String,
    pub unique_id: String,
}

// 错误类型常量
pub const ERROR_ACCESS_DENIED: &str = "ACCESS_DENIED_NEED_LOGIN";
pub const ERROR_CAPTCHA_REQUIRED: &str = "CAPTCHA_REQUIRED";
