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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_connections, get_processes, kill_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
