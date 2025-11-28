use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use tauri::api::process::{Command as TauriCommand, CommandEvent};

// 全局保存运行中的淘宝爬虫进程
lazy_static::lazy_static! {
    static ref TAOBAO_PROCESSES: Arc<Mutex<HashMap<String, (u32, AppHandle)>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Clone, serde::Serialize)]
struct TaobaoLogPayload {
    room_id: String,
    log_type: String,  // stdout, stderr, error
    message: String,
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
    app_handle: AppHandle,
) -> Result<TaobaoLiveInfo, String> {
    println!("🎯 [start_taobao_crawler] 启动淘宝爬虫");
    println!("📺 直播间ID: {}", room_id);
    if let Some(ref url) = push_url {
        println!("🔗 推送地址: {}", url);
    }

    // 检查是否已经在运行
    {
        let processes = TAOBAO_PROCESSES.lock().unwrap();
        if processes.contains_key(&room_id) {
            return Err(format!("直播间 {} 的爬虫已在运行中", room_id));
        }
    }

    // 检测 Python 命令
    let python_cmd = detect_python_command().await?;
    println!("✅ 使用 Python: {}", python_cmd);

    // 获取项目根目录（src-tauri 的父目录）
    let current_dir = std::env::current_dir().map_err(|e| format!("无法获取当前目录: {}", e))?;
    let project_root = current_dir.parent()
        .ok_or("无法找到项目根目录")?;

    // taobao_crawler.py 的绝对路径
    let script_path = project_root.join("taobao_crawler.py");

    if !script_path.exists() {
        return Err(format!("未找到 taobao_crawler.py，路径: {:?}", script_path));
    }

    println!("📂 项目根目录: {:?}", project_root);
    println!("📄 脚本路径: {:?}", script_path);

    // 构建命令参数
    let mut args = vec![
        script_path.to_string_lossy().to_string(),
        "--room_id".to_string(),
        room_id.clone(),
    ];

    if let Some(url) = push_url {
        args.push("--push_url".to_string());
        args.push(url);
    }

    println!("📝 执行命令: {} {}", python_cmd, args.join(" "));

    // 使用 Tauri Command API 启动进程并监控输出
    let room_id_clone = room_id.clone();
    let app_handle_clone = app_handle.clone();

    let (mut rx, child) = TauriCommand::new(python_cmd)
        .args(args)
        .current_dir(project_root.to_path_buf())  // 设置工作目录为项目根目录
        .spawn()
        .map_err(|e| format!("启动失败: {}. 请确保已安装 Python 和相关依赖 (pip install playwright loguru aiohttp)", e))?;

    // 获取 PID
    let pid = child.pid();
    println!("✅ 淘宝爬虫进程已启动，PID: {}", pid);

    // 保存进程信息
    {
        let mut processes = TAOBAO_PROCESSES.lock().unwrap();
        processes.insert(room_id.clone(), (pid, app_handle.clone()));
    }

    // 在后台任务中监听进程输出
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    println!("📤 [淘宝爬虫 {}] {}", room_id_clone, line);
                    let _ = app_handle_clone.emit_all("taobao-log", TaobaoLogPayload {
                        room_id: room_id_clone.clone(),
                        log_type: "stdout".to_string(),
                        message: line,
                    });
                }
                CommandEvent::Stderr(line) => {
                    println!("⚠️  [淘宝爬虫 {}] {}", room_id_clone, line);
                    let _ = app_handle_clone.emit_all("taobao-log", TaobaoLogPayload {
                        room_id: room_id_clone.clone(),
                        log_type: "stderr".to_string(),
                        message: line,
                    });
                }
                CommandEvent::Error(err) => {
                    println!("❌ [淘宝爬虫 {}] 错误: {}", room_id_clone, err);
                    let _ = app_handle_clone.emit_all("taobao-log", TaobaoLogPayload {
                        room_id: room_id_clone.clone(),
                        log_type: "error".to_string(),
                        message: err,
                    });
                }
                CommandEvent::Terminated(payload) => {
                    println!("🛑 [淘宝爬虫 {}] 进程已终止，退出码: {:?}", room_id_clone, payload.code);
                    let _ = app_handle_clone.emit_all("taobao-log", TaobaoLogPayload {
                        room_id: room_id_clone.clone(),
                        log_type: "terminated".to_string(),
                        message: format!("进程已终止，退出码: {:?}", payload.code),
                    });
                    // 从进程列表中移除
                    let mut processes = TAOBAO_PROCESSES.lock().unwrap();
                    processes.remove(&room_id_clone);
                }
                _ => {}
            }
        }
    });

    Ok(TaobaoLiveInfo {
        room_id,
        status: "running".to_string(),
        message: format!("淘宝爬虫已启动，PID: {}。浏览器窗口应该会弹出，请稍候...", pid),
    })
}

/// 停止淘宝直播间爬虫
#[tauri::command]
pub async fn stop_taobao_crawler(room_id: String) -> Result<String, String> {
    println!("🛑 [stop_taobao_crawler] 停止淘宝爬虫: {}", room_id);

    let pid = {
        let mut processes = TAOBAO_PROCESSES.lock().unwrap();
        if let Some((pid, _)) = processes.remove(&room_id) {
            pid
        } else {
            return Err(format!("未找到直播间 {} 的运行中进程", room_id));
        }
    };

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
}

/// 检查淘宝爬虫是否在运行
#[tauri::command]
pub fn check_taobao_crawler_status(room_id: String) -> bool {
    let processes = TAOBAO_PROCESSES.lock().unwrap();
    processes.contains_key(&room_id)
}

/// 检测可用的 Python 命令
async fn detect_python_command() -> Result<String, String> {
    // 尝试 python3
    if let Ok(output) = std::process::Command::new("python3")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            return Ok("python3".to_string());
        }
    }

    // 尝试 python
    if let Ok(output) = std::process::Command::new("python")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            return Ok("python".to_string());
        }
    }

    Err("未找到 Python。请安装 Python 3.7+ 并确保在系统 PATH 中".to_string())
}
