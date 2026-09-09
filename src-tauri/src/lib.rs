mod netinfo;

use netinfo::{ConnectionInfo, ProcessInfo};

/// 获取全部 TCP/UDP 连接快照
#[tauri::command]
fn get_connections() -> Vec<ConnectionInfo> {
    netinfo::get_connections()
}

/// 获取系统全部进程（含无网络连接进程）及连接统计
#[tauri::command]
fn get_processes() -> Vec<ProcessInfo> {
    netinfo::get_processes()
}

/// 结束指定 PID 的进程（taskkill /F）
#[tauri::command]
fn kill_process(pid: u32) -> Result<String, String> {
    let output = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output()
        .map_err(|e| format!("无法执行 taskkill：{}", e))?;
    if output.status.success() {
        Ok(format!("进程 {} 已结束", pid))
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = format!("{} {}", stdout.trim(), stderr.trim()).trim().to_string();
        Err(if msg.is_empty() {
            format!("结束进程 {} 失败", pid)
        } else {
            msg
        })
    }
}

/// Windows 11：为无边框窗口设置系统级圆角（DWMWA_WINDOW_CORNER_PREFERENCE）。
/// Windows 10 不支持该属性，调用会静默失败，无副作用。
#[cfg(target_os = "windows")]
fn apply_window_round(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };
    if let Ok(hwnd) = window.hwnd() {
        // tauri 内部用 windows 0.61，本项目用 0.62，用裸指针重建本版本 HWND
        let local_hwnd = HWND(hwnd.0);
        unsafe {
            let preference = DWMWCP_ROUND;
            let _ = DwmSetWindowAttribute(
                local_hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const _ as *const _,
                std::mem::size_of_val(&preference) as u32,
            );
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // CPU 占用率采用两次采样增量法（见 netinfo / ADR-005）。
    // 在窗口与 WebView 创建之前，于后台先采一次建立基线，使前端首屏加载即可显示 CPU；
    // 不引入任何定时轮询，界面数据仍只在用户手动刷新时更新。
    std::thread::spawn(|| {
        let _ = netinfo::get_processes();
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_connections, get_processes, kill_process])
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    apply_window_round(&window);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
