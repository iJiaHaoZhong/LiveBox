use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CookieStore {
    pub cookies: Vec<CookieData>,
}

impl CookieStore {
    pub fn new() -> Self {
        CookieStore {
            cookies: Vec::new(),
        }
    }

    /// 从文件加载 cookies
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let store: CookieStore = serde_json::from_str(&content)?;
        Ok(store)
    }

    /// 保存 cookies 到文件
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, json)?;
        Ok(())
    }

    /// 从 cookie 字符串解析（格式：name1=value1; name2=value2）
    pub fn from_cookie_string(cookie_str: &str, domain: &str) -> Self {
        let mut cookies = Vec::new();

        for pair in cookie_str.split(';') {
            let pair = pair.trim();
            if let Some(eq_pos) = pair.find('=') {
                let name = pair[..eq_pos].trim().to_string();
                let value = pair[eq_pos + 1..].trim().to_string();

                cookies.push(CookieData {
                    name,
                    value,
                    domain: domain.to_string(),
                    path: "/".to_string(),
                });
            }
        }

        CookieStore { cookies }
    }

    /// 转换为 cookie 字符串（格式：name1=value1; name2=value2）
    pub fn to_cookie_string(&self) -> String {
        self.cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// 获取默认的 cookie 文件路径
    pub fn get_default_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let mut path = PathBuf::from(home);
        path.push(".livebox");
        path.push("douyin_cookies.json");

        Ok(path)
    }
}
