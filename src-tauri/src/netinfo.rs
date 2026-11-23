//! 进程与网络连接采集：单次快照、IPv4/IPv6、资源状态与进程身份。
//! 设计理由见 docs/adr/0009-single-snapshot-with-metric-status.md。

use std::ffi::c_void;
use std::net::Ipv6Addr;
use std::path::Path;
use std::ptr::null_mut;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6TABLE_OWNER_PID,
    MIB_TCPTABLE_OWNER_PID, MIB_UDP6TABLE_OWNER_PID, MIB_UDPTABLE_OWNER_PID,
    TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
    TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

const AF_INET: u32 = 2;
const AF_INET6: u32 = 23;
const NO_ERROR: u32 = 0;
/// 第一次调用只传 NULL 缓冲区时返回，同时回填所需字节数
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
/// OpenProcess 拒绝访问（Windows ERROR_ACCESS_DENIED）
const ERROR_ACCESS_DENIED: u32 = 5;

/// windows-rs 把 Win32 失败包成 HRESULT_FROM_WIN32(GetLastError())，
/// 即 0x8007_0005；也兼容直接返回原始码的实现。不能只比低 16 位，
/// 否则 E_FAIL（0x8000_4005）会被误判成拒绝访问。
fn is_access_denied(err: &windows::core::Error) -> bool {
    let code = err.code().0 as u32;
    code == 0x8007_0005 || code == ERROR_ACCESS_DENIED
}

/* ---------- 对外 DTO（Rust serde camelCase 与 TS 镜像一一对应） ---------- */

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AddressFamily {
    IPv4,
    IPv6,
}

/// 指标状态：区分"确实是 0"与"读不到"（ADR-0009）
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MetricStatus {
    Available,
    Sampling,
    AccessDenied,
    ProcessExited,
    Unavailable,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotWarning {
    /// "tcp" | "udp" | "process" | "resource" | "permission"
    pub source: String,
    pub address_family: Option<AddressFamily>,
    /// Win32 错误码；非系统错误时为 None
    pub code: Option<u32>,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    /// Unix epoch 毫秒
    pub captured_at_ms: u64,
    pub processes: Vec<ProcessInfo>,
    pub connections: Vec<ConnectionInfo>,
    pub warnings: Vec<SnapshotWarning>,
    pub is_elevated: bool,
    pub duration_ms: u64,
}

/// 单条连接/端点的展示信息
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub protocol: String,
    pub address_family: AddressFamily,
    pub local_port: u16,
    /// IPv4: 1.2.3.4:80；IPv6: [::1]:80（含 scope 时为 [fe80::1%12]:443）
    pub local_address: String,
    pub remote_address: String,
    pub remote_port: u16,
    /// TCP 状态名；UDP 为 "-"
    pub state: String,
    pub process_name: String,
    pub process_path: String,
    pub software_name: String,
    pub pid: u32,
    /// 进程实例身份；拿不到时为 None（连接仍保留）
    pub creation_time_100ns: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub software: String,
    pub creation_time_100ns: Option<String>,
    pub tcp: u32,
    pub udp: u32,
    pub listening: u32,
    pub memory_bytes: Option<u64>,
    pub memory_status: MetricStatus,
    pub cpu_percent: Option<f64>,
    pub cpu_status: MetricStatus,
}

/* ---------- 进程元数据（一轮内每个 PID 只查一次） ---------- */

#[derive(Clone)]
struct ProcessMeta {
    name: String,
    path: String,
    software: String,
    creation: Option<String>,
    /// 物理内存工作集（字节）及其读取状态
    memory_bytes: Option<u64>,
    memory_status: MetricStatus,
    /// 累计 CPU 时间（100ns），供增量法使用
    cpu_total: Option<u64>,
    /// 读取 CPU 时间的底层状态；最终 cpu_status 还要结合前值决定
    cpu_read_status: MetricStatus,
}

impl ProcessMeta {
    fn unavailable(name: String) -> Self {
        ProcessMeta {
            name,
            path: String::new(),
            software: String::new(),
            creation: None,
            memory_bytes: None,
            memory_status: MetricStatus::Unavailable,
            cpu_total: None,
            cpu_read_status: MetricStatus::Unavailable,
        }
    }
}

struct CpuSnapshot {
    at: Instant,
    /// key = (PID, 创建时间)，避免 PID 复用导致错误差值（ADR-0009）
    totals: HashMap<(u32, String), u64>,
}

static CPU_LAST: Mutex<Option<CpuSnapshot>> = Mutex::new(None);

/* ---------- 统一快照入口 ---------- */

/// 当前进程是否持有管理员令牌（ADR-0010）。
pub fn is_process_elevated() -> bool {
    unsafe { windows::Win32::UI::Shell::IsUserAnAdmin().as_bool() }
}

/// 一次采集产出进程 + 连接 + 告警。进程枚举整体失败才返回 Err；
/// 单张连接表或单个进程失败只写 warning，不丢其余数据。
pub fn get_system_snapshot() -> Result<SystemSnapshot, String> {
    let started = Instant::now();
    let captured_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let mut warnings: Vec<SnapshotWarning> = Vec::new();

    // 1. ToolHelp 枚举全部进程一次（失败则整体失败）
    let procs = enumerate_processes()?;

    // 2. 每个 PID 的元数据与资源只读一次，供连接与进程列表共用
    let mut cache: HashMap<u32, ProcessMeta> = HashMap::new();
    for (pid, tool_name) in &procs {
        cache.insert(*pid, inspect_process(*pid, tool_name));
    }

    // 3. 采集四张 owner-PID 表（IPv4/IPv6 × TCP/UDP），逐表部分成功
    let mut connections: Vec<ConnectionInfo> = Vec::new();
    let mut get_meta = |pid: u32| -> ProcessMeta {
        if let Some(m) = cache.get(&pid) {
            return m.clone();
        }
        // 连接里出现 ToolHelp 快照之外的 PID：尽力补一次，也不让它拖垮整份快照
        let m = inspect_process(pid, &format!("pid{}", pid));
        cache.insert(pid, m.clone());
        m
    };
    unsafe {
        collect_tcp(AF_INET, AddressFamily::IPv4, &mut connections, &mut warnings, &mut get_meta);
        collect_tcp(AF_INET6, AddressFamily::IPv6, &mut connections, &mut warnings, &mut get_meta);
        collect_udp(AF_INET, AddressFamily::IPv4, &mut connections, &mut warnings, &mut get_meta);
        collect_udp(AF_INET6, AddressFamily::IPv6, &mut connections, &mut warnings, &mut get_meta);
    }

    // 4. 从同一份连接数组聚合进程统计
    let mut agg: HashMap<u32, (u32, u32, u32)> = HashMap::new();
    for c in &connections {
        let e = agg.entry(c.pid).or_insert((0, 0, 0));
        if c.protocol == "TCP" {
            e.0 += 1;
            if c.state == "LISTENING" {
                e.2 += 1;
            }
        } else {
            e.1 += 1;
        }
    }

    // 5. CPU 增量：与前一次快照比对（key 含创建时间）
    let now = Instant::now();
    let previous = CPU_LAST.lock().ok().and_then(|mut g| g.take());
    let logical_cores = std::thread::available_parallelism()
        .map(|n| n.get() as f64)
        .unwrap_or(1.0)
        .max(1.0);
    let mut current_totals: HashMap<(u32, String), u64> = HashMap::new();

    let mut processes: Vec<ProcessInfo> = procs
        .into_iter()
        .map(|(pid, _)| {
            let meta = cache
                .get(&pid)
                .cloned()
                .unwrap_or_else(|| ProcessMeta::unavailable(format!("pid{}", pid)));
            let (tcp, udp, listening) = agg.get(&pid).copied().unwrap_or((0, 0, 0));

            if let (Some(creation), Some(total)) = (&meta.creation, meta.cpu_total) {
                current_totals.insert((pid, creation.clone()), total);
            }
            let (cpu_percent, cpu_status) = cpu_metric(
                pid,
                meta.creation.as_deref(),
                meta.cpu_total,
                meta.cpu_read_status,
                previous.as_ref(),
                now,
                logical_cores,
            );

            ProcessInfo {
                pid,
                name: meta.name,
                path: meta.path,
                software: meta.software,
                creation_time_100ns: meta.creation,
                tcp,
                udp,
                listening,
                memory_bytes: meta.memory_bytes,
                memory_status: meta.memory_status,
                cpu_percent,
                cpu_status,
            }
        })
        .collect();

    if let Ok(mut guard) = CPU_LAST.lock() {
        *guard = Some(CpuSnapshot { at: now, totals: current_totals });
    }

    // 7. ToolHelp 快照偶尔漏掉 System (PID 4)，补一行保持视图完整
    if !processes.iter().any(|p| p.pid == 4) {
        processes.push(idle_system_process());
    }

    let is_elevated = is_process_elevated();

    Ok(SystemSnapshot {
        captured_at_ms,
        processes,
        connections,
        warnings,
        is_elevated,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

/// 进程枚举失败时返回 Err（这是唯一让整份快照失败的路径）
fn enumerate_processes() -> Result<Vec<(u32, String)>, String> {
    let mut procs: Vec<(u32, String)> = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| format!("无法创建进程快照：{}", e))?;
        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snapshot, &mut entry).is_err() {
            let _ = CloseHandle(snapshot);
            return Err("无法读取第一个进程".to_string());
        }
        loop {
            procs.push((entry.th32ProcessID, wide_string(&entry.szExeFile)));
            if Process32NextW(snapshot, &mut entry).is_err() {
                break;
            }
        }
        let _ = CloseHandle(snapshot);
    }
    Ok(procs)
}

/// 读一个进程的路径、软件名、创建时间、内存、累计 CPU 时间。
/// 一次 OpenProcess 完成全部读取；失败按错误码归类，不抛异常。
fn inspect_process(pid: u32, tool_name: &str) -> ProcessMeta {
    if pid == 0 {
        return ProcessMeta::unavailable("System Idle Process".into());
    }
    if pid == 4 {
        return ProcessMeta::unavailable("System".into());
    }

    let handle: HANDLE = match unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) } {
        Ok(h) => h,
        Err(e) => {
            let status = if is_access_denied(&e) {
                MetricStatus::AccessDenied
            } else {
                MetricStatus::ProcessExited
            };
            let mut meta = ProcessMeta::unavailable(tool_name.to_string());
            meta.memory_status = status;
            meta.cpu_read_status = status;
            return meta;
        }
    };

    let path = unsafe { query_process_path(handle) };
    let name = path
        .as_deref()
        .and_then(|p| Path::new(p).file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| tool_name.to_string());
    let software = path
        .as_deref()
        .and_then(get_software_name)
        .unwrap_or_default();

    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    let times_ok = unsafe {
        GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user).is_ok()
    };
    let (creation_str, cpu_total, cpu_read_status) = if times_ok {
        (
            Some(filetime_to_u64(&creation).to_string()),
            Some(filetime_to_u64(&kernel) + filetime_to_u64(&user)),
            MetricStatus::Available,
        )
    } else {
        (None, None, MetricStatus::Unavailable)
    };

    let (memory_bytes, memory_status) = process_memory(handle);
    let _ = unsafe { CloseHandle(handle) };

    ProcessMeta {
        name,
        path: path.unwrap_or_default(),
        software,
        creation: creation_str,
        memory_bytes,
        memory_status,
        cpu_total,
        cpu_read_status,
    }
}

fn process_memory(handle: HANDLE) -> (Option<u64>, MetricStatus) {
    let mut counters = PROCESS_MEMORY_COUNTERS::default();
    if unsafe {
        GetProcessMemoryInfo(
            handle,
            &mut counters,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
        .is_ok()
    } {
        (Some(counters.WorkingSetSize as u64), MetricStatus::Available)
    } else {
        (None, MetricStatus::Unavailable)
    }
}

fn cpu_metric(
    pid: u32,
    creation: Option<&str>,
    total_now: Option<u64>,
    read_status: MetricStatus,
    previous: Option<&CpuSnapshot>,
    now: Instant,
    logical_cores: f64,
) -> (Option<f64>, MetricStatus) {
    let (Some(creation), Some(total_now)) = (creation, total_now) else {
        return (None, read_status);
    };
    let Some(prev) = previous else {
        return (None, MetricStatus::Sampling);
    };
    let Some(total_prev) = prev.totals.get(&(pid, creation.to_string())) else {
        return (None, MetricStatus::Sampling);
    };
    let wall = now.saturating_duration_since(prev.at).as_secs_f64();
    if wall <= 0.001 {
        return (None, MetricStatus::Sampling);
    }
    let proc_secs = total_now.saturating_sub(*total_prev) as f64 * 1e-7;
    let percent = (proc_secs / (wall * logical_cores) * 100.0).clamp(0.0, 100.0);
    (Some((percent * 100.0).round() / 100.0), MetricStatus::Available)
}

fn idle_system_process() -> ProcessInfo {
    ProcessInfo {
        pid: 4,
        name: "System".into(),
        path: String::new(),
        software: String::new(),
        creation_time_100ns: None,
        tcp: 0,
        udp: 0,
        listening: 0,
        memory_bytes: None,
        memory_status: MetricStatus::Unavailable,
        cpu_percent: None,
        cpu_status: MetricStatus::Unavailable,
    }
}

/* ---------- 连接表采集 ---------- */

unsafe fn collect_tcp<F>(
    af: u32,
    family: AddressFamily,
    out: &mut Vec<ConnectionInfo>,
    warnings: &mut Vec<SnapshotWarning>,
    get_meta: &mut F,
) where
    F: FnMut(u32) -> ProcessMeta,
{
    let mut size: u32 = 0;
    let mut ret = GetExtendedTcpTable(None, &mut size, false, af, TCP_TABLE_OWNER_PID_ALL, 0);
    if ret != NO_ERROR && ret != ERROR_INSUFFICIENT_BUFFER {
        push_warning(warnings, "tcp", Some(family.clone()), ret, "TCP 连接表查询失败");
        return;
    }
    if size == 0 {
        return;
    }
    let mut buffer = vec![0u8; size as usize];
    ret = GetExtendedTcpTable(
        Some(buffer.as_mut_ptr() as *mut c_void),
        &mut size,
        false,
        af,
        TCP_TABLE_OWNER_PID_ALL,
        0,
    );
    if ret != NO_ERROR {
        push_warning(warnings, "tcp", Some(family.clone()), ret, "TCP 连接表读取失败");
        return;
    }

    if af == AF_INET6 {
        let table = buffer.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID;
        let count = (*table).dwNumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
        for row in rows {
            let local_port = port_from_network(row.dwLocalPort);
            let remote_port = port_from_network(row.dwRemotePort);
            let meta = get_meta(row.dwOwningPid);
            out.push(ConnectionInfo {
                protocol: "TCP".into(),
                address_family: family.clone(),
                local_port,
                local_address: format_ipv6(row.ucLocalAddr, row.dwLocalScopeId, local_port),
                remote_address: format_ipv6(row.ucRemoteAddr, row.dwRemoteScopeId, remote_port),
                remote_port,
                state: tcp_state_name(row.dwState),
                process_name: meta.name,
                process_path: meta.path,
                software_name: meta.software,
                pid: row.dwOwningPid,
                creation_time_100ns: meta.creation,
            });
        }
    } else {
        let table = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        let count = (*table).dwNumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
        for row in rows {
            let local_port = port_from_network(row.dwLocalPort);
            let remote_port = port_from_network(row.dwRemotePort);
            let meta = get_meta(row.dwOwningPid);
            out.push(ConnectionInfo {
                protocol: "TCP".into(),
                address_family: family.clone(),
                local_port,
                local_address: format!("{}:{}", ipv4(row.dwLocalAddr), local_port),
                remote_address: if remote_port == 0 {
                    ipv4(row.dwRemoteAddr)
                } else {
                    format!("{}:{}", ipv4(row.dwRemoteAddr), remote_port)
                },
                remote_port,
                state: tcp_state_name(row.dwState),
                process_name: meta.name,
                process_path: meta.path,
                software_name: meta.software,
                pid: row.dwOwningPid,
                creation_time_100ns: meta.creation,
            });
        }
    }
}

unsafe fn collect_udp<F>(
    af: u32,
    family: AddressFamily,
    out: &mut Vec<ConnectionInfo>,
    warnings: &mut Vec<SnapshotWarning>,
    get_meta: &mut F,
) where
    F: FnMut(u32) -> ProcessMeta,
{
    let mut size: u32 = 0;
    let mut ret = GetExtendedUdpTable(None, &mut size, false, af, UDP_TABLE_OWNER_PID, 0);
    if ret != NO_ERROR && ret != ERROR_INSUFFICIENT_BUFFER {
        push_warning(warnings, "udp", Some(family.clone()), ret, "UDP 端点表查询失败");
        return;
    }
    if size == 0 {
        return;
    }
    let mut buffer = vec![0u8; size as usize];
    ret = GetExtendedUdpTable(
        Some(buffer.as_mut_ptr() as *mut c_void),
        &mut size,
        false,
        af,
        UDP_TABLE_OWNER_PID,
        0,
    );
    if ret != NO_ERROR {
        push_warning(warnings, "udp", Some(family.clone()), ret, "UDP 端点表读取失败");
        return;
    }

    if af == AF_INET6 {
        let table = buffer.as_ptr() as *const MIB_UDP6TABLE_OWNER_PID;
        let count = (*table).dwNumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
        for row in rows {
            let local_port = port_from_network(row.dwLocalPort);
            let meta = get_meta(row.dwOwningPid);
            out.push(ConnectionInfo {
                protocol: "UDP".into(),
                address_family: family.clone(),
                local_port,
                local_address: format_ipv6(row.ucLocalAddr, row.dwLocalScopeId, local_port),
                remote_address: "-".into(),
                remote_port: 0,
                state: "-".into(),
                process_name: meta.name,
                process_path: meta.path,
                software_name: meta.software,
                pid: row.dwOwningPid,
                creation_time_100ns: meta.creation,
            });
        }
    } else {
        let table = buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
        let count = (*table).dwNumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
        for row in rows {
            let local_port = port_from_network(row.dwLocalPort);
            let meta = get_meta(row.dwOwningPid);
            out.push(ConnectionInfo {
                protocol: "UDP".into(),
                address_family: family.clone(),
                local_port,
                local_address: format!("{}:{}", ipv4(row.dwLocalAddr), local_port),
                remote_address: "-".into(),
                remote_port: 0,
                state: "-".into(),
                process_name: meta.name,
                process_path: meta.path,
                software_name: meta.software,
                pid: row.dwOwningPid,
                creation_time_100ns: meta.creation,
            });
        }
    }
}

fn push_warning(
    warnings: &mut Vec<SnapshotWarning>,
    source: &str,
    family: Option<AddressFamily>,
    code: u32,
    message: &str,
) {
    warnings.push(SnapshotWarning {
        source: source.into(),
        address_family: family,
        code: Some(code),
        message: format!("{}（错误码 {}）", message, code),
    });
}

/// WinAPI 端口字段是网络字节序，低 16 位有效
fn port_from_network(raw: u32) -> u16 {
    u16::from_be((raw & 0xFFFF) as u16)
}

fn ipv4(addr: u32) -> String {
    // dwLocalAddr 内存中为网络字节序，按小端内存字节读出即点分十进制
    let b = addr.to_le_bytes();
    format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3])
}

/// IPv6 16 字节按 IN6_ADDR 网络序解读；scope ID 以 %id 附加在地址后。
fn format_ipv6(bytes: [u8; 16], scope_id: u32, port: u16) -> String {
    let addr = Ipv6Addr::from(bytes);
    let mut s = addr.to_string();
    if scope_id != 0 {
        s.push('%');
        s.push_str(&scope_id.to_string());
    }
    format!("[{}]:{}", s, port)
}

fn tcp_state_name(state: u32) -> String {
    match state {
        2 => "LISTENING",
        3 => "SYN_SENT",
        4 => "SYN_RCVD",
        5 => "ESTABLISHED",
        6 => "FIN_WAIT1",
        7 => "FIN_WAIT2",
        8 => "CLOSE_WAIT",
        9 => "CLOSING",
        10 => "LAST_ACK",
        11 => "TIME_WAIT",
        12 => "DELETE_TCB",
        _ => "CLOSED",
    }
    .to_string()
}

/* ---------- 底层工具 ---------- */

fn filetime_to_u64(ft: &FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
}

unsafe fn query_process_path(handle: HANDLE) -> Option<String> {
    let mut size: u32 = 1024;
    let mut buf = vec![0u16; size as usize];
    let result = QueryFullProcessImageNameW(
        handle,
        PROCESS_NAME_WIN32,
        PWSTR(buf.as_mut_ptr()),
        &mut size,
    );
    if result.is_err() || size == 0 {
        return None;
    }
    buf.truncate(size as usize);
    Some(String::from_utf16_lossy(&buf))
}

fn wide_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

fn wstr(s: &str) -> (Vec<u16>, PCWSTR) {
    let v: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    let p = PCWSTR(v.as_ptr());
    (v, p)
}

/// 从 exe 版本资源读软件名（FileDescription 优先，回退 ProductName）
fn get_software_name(path: &str) -> Option<String> {
    unsafe {
        let (path16, path_pcw) = wstr(path);
        let size = GetFileVersionInfoSizeW(path_pcw, None);
        if size == 0 {
            return None;
        }
        let mut buffer = vec![0u8; size as usize];
        if GetFileVersionInfoW(path_pcw, None, size, buffer.as_mut_ptr() as *mut c_void).is_err() {
            return None;
        }
        let _ = path16;
        read_version_string(&buffer, "FileDescription")
            .or_else(|| read_version_string(&buffer, "ProductName"))
    }
}

unsafe fn read_version_string(buffer: &[u8], field: &str) -> Option<String> {
    let mut value: *mut c_void = null_mut();
    let mut len: u32 = 0;
    let (trans16, trans_pcw) = wstr("\\VarFileInfo\\Translation");
    let ok = VerQueryValueW(
        buffer.as_ptr() as *const c_void,
        trans_pcw,
        &mut value,
        &mut len,
    )
    .as_bool();
    let _ = trans16;
    if !ok || len < 4 || value.is_null() {
        return None;
    }
    let lang = *(value as *const u16);
    let codepage = *((value as *const u16).add(1));
    let subblock = format!("\\StringFileInfo\\{:04X}{:04X}\\{}", lang, codepage, field);
    let mut v2: *mut c_void = null_mut();
    let mut len2: u32 = 0;
    let (sub16, sub_pcw) = wstr(&subblock);
    let ok2 = VerQueryValueW(buffer.as_ptr() as *const c_void, sub_pcw, &mut v2, &mut len2).as_bool();
    let _ = sub16;
    if !ok2 || len2 == 0 || v2.is_null() {
        return None;
    }
    let slice = std::slice::from_raw_parts(v2 as *const u16, (len2 as usize) / 2);
    let s = String::from_utf16_lossy(slice).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/* ---------- 进程身份校验与终止（供 lib.rs 命令使用） ---------- */

/// 后端也必须拒绝的受保护进程（ADR-0008）。
/// 不能只靠前端红色确认——命令可以被直接调用；svchost 单个实例允许结束，不在此列。
const PROTECTED_PROCESS_NAMES: &[&str] = &[
    "system",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "services.exe",
    "lsass.exe",
    "winlogon.exe",
];

/// 按镜像名判断是否为核心受保护进程；读不到路径时返回 false，交给后续校验。
fn is_protected_process_name(handle: HANDLE) -> bool {
    let Some(path) = (unsafe { query_process_path(handle) }) else {
        return false;
    };
    let Some(name) = Path::new(&path).file_name() else {
        return false;
    };
    PROTECTED_PROCESS_NAMES.contains(&name.to_string_lossy().to_lowercase().as_str())
}

/// 重新读取进程当前创建时间，与期望值比对后才允许终止。
/// 返回 Err(错误码) 供前端按码显示提示。
pub fn terminate_process_checked(
    pid: u32,
    expected_creation_time_100ns: Option<&str>,
) -> Result<(), &'static str> {
    if pid == 0 || pid == 4 {
        return Err("PROTECTED_PROCESS");
    }
    let expected = expected_creation_time_100ns.unwrap_or("");
    if expected.is_empty() || expected.parse::<u64>().is_err() {
        return Err("INVALID_PROCESS_IDENTITY");
    }

    unsafe {
        let handle = match OpenProcess(
            windows::Win32::System::Threading::PROCESS_TERMINATE
                | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        ) {
            Ok(handle) => handle,
            // 权限不足与进程不存在必须分开报，否则会误导用户「进程已不存在」
            Err(e) => {
                return Err(if is_access_denied(&e) {
                    "ACCESS_DENIED"
                } else {
                    "PROCESS_NOT_FOUND"
                })
            }
        };

        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();

        if is_protected_process_name(handle) {
            let _ = CloseHandle(handle);
            return Err("PROTECTED_PROCESS");
        }
        if GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user).is_err() {
            let _ = CloseHandle(handle);
            return Err("PROCESS_NOT_FOUND");
        }
        let actual = filetime_to_u64(&creation).to_string();
        if actual != expected {
            let _ = CloseHandle(handle);
            return Err("PROCESS_IDENTITY_CHANGED");
        }

        // 在同一个已校验的 HANDLE 上终止，关闭校验后再按 PID 找进程的竞态窗口
        let ok = windows::Win32::System::Threading::TerminateProcess(handle, 1).is_ok();
        let _ = CloseHandle(handle);
        if ok { Ok(()) } else { Err("TERMINATION_FAILED") }
    }
}

/* ---------- 测试 ---------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_byte_order() {
        // WinAPI 端口字段是"网络字节序的低16位按小端读成的值"：
        // 端口 80 = 0x0050，网络序内存 [00,50] 按小端读为 0x5000
        assert_eq!(port_from_network(0x0000_5000), 80);
        // 端口 8080 = 0x1F90，网络序内存 [1F,90] 按小端读为 0x901F
        assert_eq!(port_from_network(0x0000_901F), 8080);
    }

    #[test]
    fn ipv4_format() {
        // dwLocalAddr 在内存里是网络字节序，CPU 按小端解读成一个 u32，
        // 再由 ipv4() 用 to_le_bytes 拆回原字节序。用回环地址避免测试含私有网段字面量。
        let addr = u32::from_le_bytes([127, 0, 0, 1]);
        assert_eq!(ipv4(addr), "127.0.0.1");
    }

    #[test]
    fn ipv6_format_with_and_without_scope() {
        let mut bytes = [0u8; 16];
        bytes[15] = 1; // ::1
        assert_eq!(format_ipv6(bytes, 0, 8080), "[::1]:8080");
        assert_eq!(format_ipv6(bytes, 12, 443), "[::1%12]:443");
    }

    #[test]
    fn metric_status_separates_zero_from_unavailable() {
        let available = (Some(0u64), MetricStatus::Available);
        let unavailable = (None, MetricStatus::Unavailable);
        assert_ne!(available, unavailable);
    }

    #[test]
    fn snapshot_is_internally_consistent() {
        let snap = match get_system_snapshot() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("跳过：进程枚举不可用（{}）", e);
                return;
            }
        };
        let sum_tcp: u32 = snap.processes.iter().map(|p| p.tcp).sum();
        let sum_udp: u32 = snap.processes.iter().map(|p| p.udp).sum();
        assert_eq!(
            sum_tcp,
            snap.connections.iter().filter(|c| c.protocol == "TCP").count() as u32
        );
        assert_eq!(
            sum_udp,
            snap.connections.iter().filter(|c| c.protocol == "UDP").count() as u32
        );
        assert!(snap.connections.iter().all(|c| !c.local_address.is_empty()));
        assert!(snap.connections.iter().all(|c| matches!(c.address_family, AddressFamily::IPv4 | AddressFamily::IPv6)));
        assert!(snap.processes.iter().any(|p| p.pid == 4));
        println!(
            "快照：进程 {}，连接 {}（IPv6 {}），warnings {}，elevated {}，耗时 {}ms",
            snap.processes.len(),
            snap.connections.len(),
            snap.connections.iter().filter(|c| matches!(c.address_family, AddressFamily::IPv6)).count(),
            snap.warnings.len(),
            snap.is_elevated,
            snap.duration_ms
        );
    }
}
