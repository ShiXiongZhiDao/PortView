/** Rust 侧 AddressFamily 的镜像 */
export type AddressFamily = "IPv4" | "IPv6";

/** Rust 侧 MetricStatus 的镜像：区分"确实是 0"与"读不到" */
export type MetricStatus =
  | "available"
  | "sampling"
  | "accessDenied"
  | "processExited"
  | "unavailable";

/** 单次快照中某一采集源的非致命失败 */
export interface SnapshotWarning {
  /** "tcp" | "udp" | "process" | "resource" | "permission" */
  source: string;
  addressFamily: AddressFamily | null;
  /** Win32 错误码；非系统错误时为 null */
  code: number | null;
  message: string;
}

/** 一次采集的完整结果（Rust SystemSnapshot 镜像） */
export interface SystemSnapshot {
  /** Unix epoch 毫秒 */
  capturedAtMs: number;
  processes: ProcessInfo[];
  connections: ConnectionInfo[];
  warnings: SnapshotWarning[];
  isElevated: boolean;
  durationMs: number;
}

/** Rust 侧 ConnectionInfo 的镜像（serde camelCase） */
export interface ConnectionInfo {
  protocol: "TCP" | "UDP" | string;
  addressFamily: AddressFamily;
  localPort: number;
  localAddress: string;
  remoteAddress: string;
  remotePort: number;
  state: string;
  processName: string;
  processPath: string;
  softwareName: string;
  pid: number;
  /** 进程实例身份；拿不到时为 null */
  creationTime100ns: string | null;
}

/** Rust 侧 ProcessInfo 的镜像（serde camelCase） */
export interface ProcessInfo {
  pid: number;
  name: string;
  path: string;
  software: string;
  creationTime100ns: string | null;
  tcp: number;
  udp: number;
  listening: number;
  memoryBytes: number | null;
  memoryStatus: MetricStatus;
  cpuPercent: number | null;
  cpuStatus: MetricStatus;
}
