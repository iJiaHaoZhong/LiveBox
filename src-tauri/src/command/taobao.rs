use std::process::{Command, Stdio};
use tauri::api::process::{Command as TauriCommand, CommandEvent};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::Manager;

// 全局保存运行中的淘宝爬虫进程
lazy_static::lazy_static! {
    static ref TAOBAO_PROCESSES: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaobaoLiveInfo {
    pub room_id: String,
    pub status: String,
    pub message: String,
}

/// 启动淘宝直播间爬虫
///
/// 参数:
/// - room_id: 淘宝直播间ID
/// - push_url: 推送弹幕消息的URL（可选）
#[tauri::command]
pub async fn start_taobao_crawler(
    room_id: String,
    push_url: Option<String>,
) -> Result<TaobaoLiveInfo, String> {
    println!("🎯 [start_taobao_crawler] 启动淘宝爬虫");
    println!("📺 直播间ID: {}", room_id);
    if let Some(ref url) = push_url {
        println!("🔗 推送地址: {}", url);
    }

    // 检查Python是否可用
    let python_check = Command::new("python3")
        .arg("--version")
        .output();

    if python_check.is_err() {
        println!("⚠️  python3 不可用，尝试使用 python");
    }

    // 构建Python命令
    let python_cmd = if python_check.is_ok() { "python3" } else { "python" };

    // 构建命令参数
    let mut args = vec![
        "taobao_crawler.py".to_string(),
        "--room_id".to_string(),
        room_id.clone(),
    ];

    if let Some(url) = push_url {
        args.push("--push_url".to_string());
        args.push(url);
    }

    println!("📝 执行命令: {} {}", python_cmd, args.join(" "));

    // 启动Python进程（异步，非阻塞）
    match Command::new(python_cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            let pid = child.id();
            println!("✅ 淘宝爬虫进程已启动，PID: {}", pid);

            // 保存进程ID
            let mut processes = TAOBAO_PROCESSES.lock().unwrap();
            processes.insert(room_id.clone(), pid);

            Ok(TaobaoLiveInfo {
                room_id,
                status: "running".to_string(),
                message: format!("淘宝爬虫已启动，进程ID: {}", pid),
            })
        }
        Err(e) => {
            println!("❌ 启动淘宝爬虫失败: {}", e);
            Err(format!("启动失败: {}. 请确保已安装Python和相关依赖(pip install playwright loguru)", e))
        }
    }
}

/// 停止淘宝直播间爬虫
#[tauri::command]
pub async fn stop_taobao_crawler(room_id: String) -> Result<String, String> {
    println!("🛑 [stop_taobao_crawler] 停止淘宝爬虫: {}", room_id);

    let mut processes = TAOBAO_PROCESSES.lock().unwrap();

    if let Some(pid) = processes.remove(&room_id) {
        println!("📋 找到进程 PID: {}", pid);

        // 尝试优雅地终止进程
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            match signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => {
                    println!("✅ 已发送终止信号到进程 {}", pid);
                    Ok(format!("淘宝爬虫已停止 (PID: {})", pid))
                }
                Err(e) => {
                    println!("⚠️  发送终止信号失败: {}", e);
                    Err(format!("停止失败: {}", e))
                }
            }
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            match Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F"])
                .output()
            {
                Ok(_) => {
                    println!("✅ 已终止进程 {}", pid);
                    Ok(format!("淘宝爬虫已停止 (PID: {})", pid))
                }
                Err(e) => {
                    println!("⚠️  终止进程失败: {}", e);
                    Err(format!("停止失败: {}", e))
                }
            }
        }
    } else {
        println!("⚠️  未找到运行中的爬虫进程");
        Err(format!("未找到直播间 {} 的运行中进程", room_id))
    }
}

/// 检查淘宝爬虫是否在运行
#[tauri::command]
pub fn check_taobao_crawler_status(room_id: String) -> bool {
    let processes = TAOBAO_PROCESSES.lock().unwrap();
    processes.contains_key(&room_id)
}
