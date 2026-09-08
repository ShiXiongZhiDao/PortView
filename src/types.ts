/** Rust 侧 ConnectionInfo 的镜像类型（serde camelCase） */
export interface ConnectionInfo {
  protocol: "TCP" | "UDP" | string;
  localPort: number;
  localAddress: string;
  remoteAddress: string;
  remotePort: number;
  state: string;
  processName: string;
  processPath: string;
  softwareName: string;
  pid: number;
}
