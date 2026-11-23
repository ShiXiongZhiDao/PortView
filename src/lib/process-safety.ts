/**
 * 危险进程判定：仅用于结束前的红色二次确认，不是后端的硬拒绝名单。
 * 后端 PROTECTED_PROCESS_NAMES 只拦 Windows 核心受保护进程；
 * 这里额外包含 svchost 等"结束可能影响系统"的进程，所以两处名单刻意不同。
 */
const CRITICAL_NAMES = new Set([
  "system",
  "svchost.exe",
  "csrss.exe",
  "wininit.exe",
  "services.exe",
  "lsass.exe",
]);

export function isCriticalProcess(pid: number, name: string): boolean {
  return pid <= 4 || CRITICAL_NAMES.has(name.toLowerCase());
}
