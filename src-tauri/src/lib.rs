mod netinfo;

use netinfo::SystemSnapshot;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

/// 当前界面语言（"zh"/"en"），用于系统托盘菜单文案同步
static LANG: Mutex<&'static str> = Mutex::new("zh");

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchState {
    from_elevation: bool,
    is_elevated: bool,
    state: Option<String>,
}

/// 读取本程序自己写入的 `flag value` 启动参数（ADR-0010）
fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn launched_elevated() -> bool {
    std::env::args().any(|a| a == "--elevated")
}

/// PCWSTR 需要以 NUL 结尾的 UTF-16 缓冲区
#[cfg(target_os = "windows")]
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 获取单次统一快照：进程 + 连接 + 告警 + 能力状态（见 ADR-0009）。
/// 前端只调用这一条命令，一轮刷新只采集一份 TCP/UDP 数据。
#[tauri::command]
fn get_system_snapshot() -> Result<SystemSnapshot, String> {
    netinfo::get_system_snapshot()
}

/// 启动状态：前端据此恢复提权前的视图与筛选（ADR-0010）。
#[tauri::command]
fn get_launch_state() -> LaunchState {
    LaunchState {
        from_elevation: launched_elevated(),
        is_elevated: netinfo::is_process_elevated(),
        state: arg_value("--state"),
    }
}

/// 从受限模式重启为管理员实例。返回 true 表示已创建提权进程；
/// false 表示当前进程本来就是管理员（例如快照尚未回来时提前点击）。
/// 调用方只有在 true 时才应调用 `finish_elevation` 退出当前实例。
#[tauri::command]
fn relaunch_elevated(app: tauri::AppHandle, state: Option<String>) -> Result<bool, String> {
    if netinfo::is_process_elevated() {
        return Ok(false);
    }

    let exe = std::env::current_exe().map_err(|_| "ELEVATION_FAILED".to_string())?;
    let mut params = format!("--elevated --wait-pid {}", std::process::id());
    if let Some(state) = state.filter(|s| !s.is_empty()) {
        // 前端传 base64url，无空格与引号，可直接拼进参数串
        params.push_str(&format!(" --state {state}"));
    }
    if let Some(window) = app.get_webview_window("main") {
        if let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) {
            params.push_str(&format!(
                " --bounds {},{},{},{}",
                pos.x, pos.y, size.width, size.height
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        use windows::core::{w, PCWSTR};
        use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_CANCELLED};
        use windows::Win32::UI::Shell::{
            ShellExecuteExW, SEE_MASK_FLAG_NO_UI, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
        };
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let file = wide(&exe.to_string_lossy());
        let parameters = wide(&params);
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_FLAG_NO_UI,
            lpVerb: w!("runas"),
            lpFile: PCWSTR(file.as_ptr()),
            lpParameters: PCWSTR(parameters.as_ptr()),
            nShow: SW_SHOWNORMAL.0,
            ..Default::default()
        };

        unsafe {
            if let Err(err) = ShellExecuteExW(&mut info) {
                let last = GetLastError();
                // UAC 取消可能落在 GetLastError()=1223，也可能只体现在 HRESULT 上
                let cancelled = last == ERROR_CANCELLED || err.code().0 as u32 == 0x8007_04C7;
                return Err(if cancelled {
                    "ELEVATION_CANCELLED".into()
                } else {
                    format!("ELEVATION_FAILED:{}", last.0)
                });
            }
            if !info.hProcess.is_invalid() {
                let _ = CloseHandle(info.hProcess);
            }
        }
        return Ok(true);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = exe;
        return Err("ELEVATION_UNSUPPORTED".into());
    }
}

/// 提权实例已经创建：隐藏旧实例的托盘图标后退出，避免双托盘。
#[tauri::command]
fn finish_elevation(app: tauri::AppHandle) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_visible(false);
    }
    app.exit(0);
}

/// 结束指定进程：先校验进程实例身份（PID + 创建时间），再在同一 HANDLE 上终止（ADR-0008）。
/// 成功返回空；失败返回结构化错误码（ACCESS_DENIED / PROCESS_NOT_FOUND /
/// PROTECTED_PROCESS / INVALID_PROCESS_IDENTITY / PROCESS_IDENTITY_CHANGED / TERMINATION_FAILED）。
/// expected_creation_time_100ns 为十进制字符串，与快照中的 creationTime100ns 一致。
#[tauri::command]
fn kill_process(pid: u32, expected_creation_time_100ns: Option<String>) -> Result<(), String> {
    netinfo::terminate_process_checked(pid, expected_creation_time_100ns.as_deref())
        .map_err(|code| code.to_string())
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
/// 判据是 exe 同目录是否存在 `uninstall.exe`：NSIS 安装时一定会写这个文件，
/// 便携版 zip 只有 exe + README，没有它。前端据此禁用「下载并安装」——
/// Tauri updater 在 Windows 上只能执行安装包，便携 exe 没有被安装器替换的目标目录。
///
/// 不用「exe 是否位于当前用户 LOCALAPPDATA\PortView」来判：应用带 requireAdministrator
/// manifest，标准用户启动时会 UAC 提权切换到管理员账户，进程的 LOCALAPPDATA 随之改变，
/// 而 exe 仍在原用户目录，前缀对不上会把安装版误判成便携版（0.2.0 实测踩到）。
/// 用户若手动把便携版 exe 连同 uninstall.exe 放到同一目录会被误判为安装版，
/// 这是可接受的边界情况。
/// 判据本体，接受 exe 路径，便于回归测试。
fn is_installed_build_at(exe: &std::path::Path) -> bool {
    match exe.parent() {
        Some(dir) => dir.join("uninstall.exe").exists(),
        None => false,
    }
}

#[tauri::command]
fn is_installed_build() -> bool {
    match std::env::current_exe() {
        Ok(exe) => is_installed_build_at(&exe),
        Err(_) => false,
    }
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

/// 提权实例创建窗口前等旧实例退出，避免短暂双托盘（ADR-0010）。
/// 返回 true 表示旧实例已退出或本来就不存在，可以安全注册单实例插件。
#[cfg(target_os = "windows")]
fn wait_for_previous_instance() -> bool {
    let Some(pid) = arg_value("--wait-pid").and_then(|v| v.parse::<u32>().ok()) else {
        return true;
    };
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE};
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_SYNCHRONIZE, false, pid) {
            let result = WaitForSingleObject(handle, 5000);
            let _ = CloseHandle(handle);
            return result == WAIT_OBJECT_0;
        }
    }
    true
}

/// 提权接管时把窗口放回原来的位置与大小。
#[cfg(target_os = "windows")]
fn apply_launch_bounds(window: &tauri::WebviewWindow) {
    let Some(raw) = arg_value("--bounds") else {
        return;
    };
    let nums: Vec<i64> = raw.split(',').filter_map(|s| s.parse().ok()).collect();
    if nums.len() != 4 {
        return;
    }
    let width = nums[2].clamp(1024, 10000) as u32;
    let height = nums[3].clamp(640, 10000) as u32;
    let _ = window.set_position(tauri::PhysicalPosition::new(nums[0] as i32, nums[1] as i32));
    let _ = window.set_size(tauri::PhysicalSize::new(width, height));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 提权接管：先等旧实例退出，再建窗口与托盘（ADR-0010）
    #[cfg(target_os = "windows")]
    let previous_exited = wait_for_previous_instance();
    #[cfg(not(target_os = "windows"))]
    let previous_exited = true;

    // CPU 占用率采用两次采样增量法（采样 key 含进程创建时间，见 netinfo / ADR-0009）。
    // 在窗口与 WebView 创建之前，于后台先采一次建立基线，使前端首屏加载即可显示 CPU；
    // 这只是采样基线，不是界面定时轮询——界面的定时刷新由前端 timer 按 ADR-0007 控制。
    std::thread::spawn(|| {
        let _ = netinfo::get_system_snapshot();
    });

    // 单实例（防多开）必须作为第一个插件注册（见 docs/ADR-006）。
    // 提权接管实例要等旧实例退出后再注册，才能接管互斥体；等待超时才跳过，
    // 以免被仍在运行的旧实例直接吞掉（ADR-0010）。
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        if !launched_elevated() || previous_exited {
            builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }));
        }
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
            get_system_snapshot,
            get_launch_state,
            relaunch_elevated,
            finish_elevation,
            kill_process,
            set_language,
            open_in_explorer,
            is_installed_build
        ])
        .setup(|app| {
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                apply_window_round(&window);
                apply_launch_bounds(&window);
            }
            let _ = setup_tray(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::is_installed_build_at;

    #[test]
    fn installed_build_keyed_off_sibling_uninstaller() {
        use std::fs;
        use std::path::PathBuf;

        let root = std::env::temp_dir().join(format!("pv-install-kind-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let app_dir = root.join("PortView");
        fs::create_dir_all(&app_dir).unwrap();
        let exe = app_dir.join("portview.exe");
        fs::write(&exe, b"exe").unwrap();

        // 便携版：只有 exe，无同目录 uninstall.exe → false
        assert!(!is_installed_build_at(&exe));
        // 安装版：NSIS 会写 uninstall.exe → true
        fs::write(app_dir.join("uninstall.exe"), b"uninst").unwrap();
        assert!(is_installed_build_at(&exe));
        // 无父目录的路径 → false
        assert!(!is_installed_build_at(&PathBuf::from("portview.exe")));

        let _ = fs::remove_dir_all(&root);
    }
}