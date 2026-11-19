mod netinfo;

use netinfo::{ConnectionInfo, ProcessInfo};
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

/// 当前界面语言（"zh"/"en"），用于系统托盘菜单文案同步
static LANG: Mutex<&'static str> = Mutex::new("zh");

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

/// 将命令输出字节按系统 ANSI 代码页解码为字符串。
///
/// `taskkill` 等控制台工具的输出是 ANSI/OEM 编码（中文 Windows 为 GBK/cp936），
/// 直接 `String::from_utf8_lossy` 会把 GBK 字节当 UTF-8 解码成乱码。
/// 用 `GetACP` 取系统实际代码页 + `MultiByteToWideChar` 转 UTF-16，任何区域设置均正确。
fn decode_ansi_lossy(bytes: &[u8]) -> String {
    use windows::Win32::Globalization::{GetACP, MultiByteToWideChar};

    if bytes.is_empty() {
        return String::new();
    }
    unsafe {
        let cp = GetACP();
        // 第一段参数 None：只求转换后的 UTF-16 字符数（windows 0.62 的 slice 版签名）
        let needed = MultiByteToWideChar(cp, Default::default(), bytes, None);
        if needed <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut buf = vec![0u16; needed as usize];
        let written = MultiByteToWideChar(cp, Default::default(), bytes, Some(&mut buf));
        buf.truncate(written.max(0) as usize);
        String::from_utf16_lossy(&buf)
    }
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
        let stdout = decode_ansi_lossy(&output.stdout);
        let stderr = decode_ansi_lossy(&output.stderr);
        let msg = format!("{} {}", stdout.trim(), stderr.trim()).trim().to_string();
        Err(if msg.is_empty() {
            format!("结束进程 {} 失败", pid)
        } else {
            msg
        })
    }
}

/// 设置界面语言，并同步重建系统托盘菜单文案（"zh"/"en"）
#[tauri::command]
fn set_language(app: tauri::AppHandle, lang: String) {
    if let Ok(mut g) = LANG.lock() {
        *g = if lang == "en" { "en" } else { "zh" };
    }
    let _ = rebuild_tray(&app);
}

/// 在资源管理器中定位并选中程序文件（右键菜单"打开程序地址"）
///
/// 必须用 `raw_arg` 原样传递 `/select,"路径"`：explorer 依赖该精确语法（含引号），
/// 若用普通 `.arg()` 传参，Rust 会再次转义/加引号，导致只打开文件管理器而不选中文件。
#[tauri::command]
fn open_in_explorer(path: String) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    let arg = format!("/select,\"{}\"", path);
    std::process::Command::new("explorer")
        .raw_arg(arg)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("无法打开资源管理器：{}", e))
}

/// 当前运行的是否是安装版（见 ADR-0002）。
///
/// NSIS 用 currentUser 安装到 `%LOCALAPPDATA%\PortView`。便携版解压到别处时返回 false，
/// 前端据此禁用「下载并安装」——Tauri updater 在 Windows 上只能执行安装包，
/// 便携 exe 没有被安装器替换的目标目录。
/// 用户若手动把便携版解压到安装目录会被误判为安装版，这是可接受的边界情况。
#[tauri::command]
fn is_installed_build() -> bool {
    let exe = match std::env::current_exe() {
        Ok(path) => path.to_string_lossy().to_lowercase(),
        Err(_) => return false,
    };
    let local = match std::env::var_os("LOCALAPPDATA") {
        Some(value) => value.to_string_lossy().to_lowercase(),
        None => return false,
    };
    exe.starts_with(&format!("{}\\portview\\", local.trim_end_matches('\\')))
}

/* ---------- 系统托盘 ---------- */

/// 切换主窗口显示/隐藏（托盘左键单击）
fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

/// 显示并聚焦主窗口（托盘菜单"显示"）
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 按当前语言生成托盘菜单文案
fn tray_labels() -> (&'static str, &'static str) {
    let lang = LANG.lock().map(|g| *g).unwrap_or("zh");
    if lang == "en" {
        ("Show", "Quit")
    } else {
        ("显示", "退出")
    }
}

/// 重建托盘菜单（语言切换后调用，实现中英同步）
fn rebuild_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let (show_text, quit_text) = tray_labels();
    let show = MenuItem::with_id(app, "show", show_text, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

/// 创建系统托盘（图标 + 菜单 + 左键切换窗口）
fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let Some(icon) = app.default_window_icon() else {
        return Ok(());
    };
    // 注意：必须显式指定 id "main"，否则 tray-icon 默认用随机唯一 id，
    // 会导致下方 rebuild_tray 里 app.tray_by_id("main") 取不到托盘、菜单挂不上。
    TrayIconBuilder::with_id("main")
        .icon(icon.clone())
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    rebuild_tray(app)?;
    Ok(())
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

    // 单实例（防多开）必须作为第一个插件注册（见 docs/ADR-006）。
    // 第二实例启动时新进程立即退出，回调在已运行的实例里执行：
    // 把主窗口从托盘隐藏态/最小化态唤起并聚焦，符合"托盘常驻 + 防多开"的产品形态。
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // 匿名使用统计（ADR-0003）。App Key 由构建环境注入；未配置时传空串，
        // 插件会静默丢掉所有事件，开源仓库与本地 clone 都不需要藏 key。
        .plugin(
            tauri_plugin_aptabase::Builder::new(
                option_env!("PORTVIEW_APTABASE_KEY").unwrap_or(""),
            )
            .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_connections,
            get_processes,
            kill_process,
            set_language,
            open_in_explorer,
            is_installed_build
        ])
        .setup(|app| {
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                apply_window_round(&window);
            }
            let _ = setup_tray(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::decode_ansi_lossy;

    #[test]
    fn decodes_ansi_bytes() {
        let cp = unsafe { windows::Win32::Globalization::GetACP() };
        if cp != 936 {
            eprintln!("跳过：系统 ANSI 代码页 {} 非 GBK/936", cp);
            return;
        }
        // "错误: 进程 25476 未找到。" 的 GBK/cp936 字节（与中文 Windows 上 taskkill 输出一致）
        let gbk: [u8; 25] = [
            180, 237, 206, 243, 58, 32, 189, 248, 179, 204, 32, 50, 53, 52, 55, 54, 32, 206,
            180, 213, 210, 181, 189, 161, 163,
        ];
        assert_eq!(decode_ansi_lossy(&gbk), "错误: 进程 25476 未找到。");
    }

    #[test]
    fn passes_through_ascii_and_empty() {
        assert_eq!(decode_ansi_lossy(b"ERROR: not found."), "ERROR: not found.");
        assert_eq!(decode_ansi_lossy(b""), "");
    }
}