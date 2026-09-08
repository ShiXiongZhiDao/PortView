//! 网络连接采集：WinAPI 枚举 TCP/UDP 连接及属主进程信息。

use std::ffi::c_void;
use std::path::Path;
use std::ptr::null_mut;

use serde::Serialize;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCPTABLE_OWNER_PID, MIB_UDPTABLE_OWNER_PID,
    TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

const AF_INET: u32 = 2;
const NO_ERROR: u32 = 0;
/// GetExtendedXxxTable 第一次调用（只传 NULL 缓冲区）时返回该值并回填所需字节数
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

/// 单条连接/端点的展示信息
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    /// "TCP" / "UDP"
    pub protocol: String,
    /// 本地端口
    pub local_port: u16,
    /// 本地地址，格式 `IP:端口`（如 `0.0.0.0:135`）
    pub local_address: String,
    /// 远程地址：TCP 为 `IP:端口`（监听状态为 `0.0.0.0`），UDP 为 `-`
    pub remote_address: String,
    /// 远程端口（TCP 监听时为 0，UDP 为 0）
    pub remote_port: u16,
    /// 状态：TCP 状态名；UDP 为 `-`
    pub state: String,
    /// 进程名（如 `svchost.exe`）
    pub process_name: String,
    /// 进程完整路径（获取失败为空）
    pub process_path: String,
    /// 软件名（exe 版本资源的 FileDescription/ProductName，获取失败为空）
    pub software_name: String,
    /// 进程 PID
    pub pid: u32,
}

/// 获取全部 TCP + UDP 连接快照
pub fn get_connections() -> Vec<ConnectionInfo> {
    let mut result = Vec::new();
    // 进程信息按 PID 缓存，避免对同一进程重复查询
    let mut cache: std::collections::HashMap<u32, (String, String, String)> =
        std::collections::HashMap::new();
    let mut get_proc = |pid: u32| -> (String, String, String) {
        cache
            .entry(pid)
            .or_insert_with(|| process_info(pid))
            .clone()
    };

    unsafe {
        collect_tcp(&mut result, &mut get_proc);
        collect_udp(&mut result, &mut get_proc);
    }
    result
}

unsafe fn collect_tcp<F>(out: &mut Vec<ConnectionInfo>, get_proc: &mut F)
where
    F: FnMut(u32) -> (String, String, String),
{
    let mut size: u32 = 0;
    let mut ret = GetExtendedTcpTable(
        None,
        &mut size,
        false,
        AF_INET,
        TCP_TABLE_OWNER_PID_ALL,
        0,
    );
    // 第一次调用以 NULL 获取所需大小，返回 ERROR_INSUFFICIENT_BUFFER 是正常路径
    if ret != NO_ERROR && ret != ERROR_INSUFFICIENT_BUFFER {
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
        AF_INET,
        TCP_TABLE_OWNER_PID_ALL,
        0,
    );
    if ret != NO_ERROR {
        return;
    }
    let table = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
    let count = (*table).dwNumEntries as usize;
    let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
    for row in rows {
        let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
        let remote_port = u16::from_be((row.dwRemotePort & 0xFFFF) as u16);
        let (pname, ppath, software) = get_proc(row.dwOwningPid);
        out.push(ConnectionInfo {
            protocol: "TCP".into(),
            local_port,
            local_address: format!("{}:{}", ipv4(row.dwLocalAddr), local_port),
            remote_address: if remote_port == 0 {
                ipv4(row.dwRemoteAddr)
            } else {
                format!("{}:{}", ipv4(row.dwRemoteAddr), remote_port)
            },
            remote_port,
            state: tcp_state_name(row.dwState),
            process_name: pname,
            process_path: ppath,
            software_name: software,
            pid: row.dwOwningPid,
        });
    }
}

unsafe fn collect_udp<F>(out: &mut Vec<ConnectionInfo>, get_proc: &mut F)
where
    F: FnMut(u32) -> (String, String, String),
{
    let mut size: u32 = 0;
    let mut ret = GetExtendedUdpTable(None, &mut size, false, AF_INET, UDP_TABLE_OWNER_PID, 0);
    if ret != NO_ERROR && ret != ERROR_INSUFFICIENT_BUFFER {
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
        AF_INET,
        UDP_TABLE_OWNER_PID,
        0,
    );
    if ret != NO_ERROR {
        return;
    }
    let table = buffer.as_ptr() as *const MIB_UDPTABLE_OWNER_PID;
    let count = (*table).dwNumEntries as usize;
    let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);
    for row in rows {
        let local_port = u16::from_be((row.dwLocalPort & 0xFFFF) as u16);
        let (pname, ppath, software) = get_proc(row.dwOwningPid);
        out.push(ConnectionInfo {
            protocol: "UDP".into(),
            local_port,
            local_address: format!("{}:{}", ipv4(row.dwLocalAddr), local_port),
            remote_address: "-".into(),
            remote_port: 0,
            state: "-".into(),
            process_name: pname,
            process_path: ppath,
            software_name: software,
            pid: row.dwOwningPid,
        });
    }
}

fn ipv4(addr: u32) -> String {
    // dwLocalAddr 内存中为网络字节序的 in_addr（如 [192,168,0,3]），
    // 直接按小端内存字节读出即为正确的点分十进制
    let b = addr.to_le_bytes();
    format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3])
}

/// TCP 状态码 → 展示名（netstat 风格）
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

/// 根据 PID 获取（进程名, 完整路径, 软件名）
fn process_info(pid: u32) -> (String, String, String) {
    if pid == 0 {
        return ("System Idle Process".into(), String::new(), String::new());
    }
    if pid == 4 {
        return ("System".into(), String::new(), String::new());
    }
    let path = get_process_path(pid);
    let name = path
        .as_ref()
        .and_then(|p| Path::new(p).file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("pid{}", pid));
    let software = path
        .as_ref()
        .and_then(|p| get_software_name(p))
        .unwrap_or_default();
    (name, path.unwrap_or_default(), software)
}

/// 通过 OpenProcess + QueryFullProcessImageNameW 获取进程完整路径
fn get_process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut size: u32 = 1024;
        let mut buf = vec![0u16; size as usize];
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(handle);
        if result.is_err() || size == 0 {
            return None;
        }
        buf.truncate(size as usize);
        Some(String::from_utf16_lossy(&buf))
    }
}

/// &str → (UTF-16 缓冲, PCWSTR)。缓冲需存活到 API 调用结束。
fn wstr(s: &str) -> (Vec<u16>, PCWSTR) {
    let v: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
    let p = PCWSTR(v.as_ptr());
    (v, p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_connections() {
        let conns = get_connections();
        assert!(!conns.is_empty(), "应至少采集到一条连接");
        assert!(conns.iter().all(|c| !c.local_address.is_empty()));
        assert!(conns.iter().all(|c| c.protocol == "TCP" || c.protocol == "UDP"));
        assert!(conns.iter().all(|c| c.state != "UNKNOWN"));
        for c in conns.iter().take(8) {
            println!(
                "{} {:>6} {:<22} {:<22} {:<10} {:<16} pid={:<7} sw={} path={}",
                c.protocol,
                c.local_port,
                c.local_address,
                c.remote_address,
                c.state,
                c.process_name,
                c.pid,
                c.software_name,
                c.process_path
            );
        }
        let tcp = conns.iter().filter(|c| c.protocol == "TCP").count();
        let udp = conns.iter().filter(|c| c.protocol == "UDP").count();
        println!("合计 {} 条：TCP={} UDP={}", conns.len(), tcp, udp);
    }
}

/// 从 exe 版本资源读取软件名（FileDescription 优先，回退 ProductName）
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

/// 按翻译表读取指定字段（如 FileDescription）
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
    let subblock = format!(
        "\\StringFileInfo\\{:04X}{:04X}\\{}",
        lang, codepage, field
    );
    let mut v2: *mut c_void = null_mut();
    let mut len2: u32 = 0;
    let (sub16, sub_pcw) = wstr(&subblock);
    let ok2 = VerQueryValueW(
        buffer.as_ptr() as *const c_void,
        sub_pcw,
        &mut v2,
        &mut len2,
    )
    .as_bool();
    let _ = sub16;
    if !ok2 || len2 == 0 || v2.is_null() {
        return None;
    }
    let slice = std::slice::from_raw_parts(v2 as *const u16, (len2 as usize) / 2);
    let s = String::from_utf16_lossy(slice).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
