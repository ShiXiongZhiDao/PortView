/**
 * 提权接管时的界面状态传递（ADR-0010）。
 *
 * 同一账户提权时 WebView2 的 localStorage 共享，主题/语言/列宽自然保留；
 * 跨账户提权会切换 Windows 用户配置，只能用启动参数带过去视图、搜索与筛选。
 * 参数走 base64url，避免空格、引号与中文在命令行里被拆开。
 */
import type { FamilyFilter, ProtocolFilter, StatusFilter } from "../stores/connections";

export interface LaunchState {
  fromElevation: boolean;
  isElevated: boolean;
  state: string | null;
}

export interface ElevatedUiState {
  view: "process" | "port";
  search: string;
  protocol: ProtocolFilter;
  status: StatusFilter;
  family: FamilyFilter;
  listeningOnly: boolean;
}

const PROTOCOLS: ProtocolFilter[] = ["全部", "TCP", "UDP"];
const STATUSES: StatusFilter[] = ["全部", "LISTENING", "ESTABLISHED", "TIME_WAIT", "CLOSE_WAIT", "其他"];
const FAMILIES: FamilyFilter[] = ["全部", "IPv4", "IPv6"];

function toBase64Url(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function fromBase64Url(encoded: string): string {
  const padded = encoded.replace(/-/g, "+").replace(/_/g, "/").padEnd(Math.ceil(encoded.length / 4) * 4, "=");
  const binary = atob(padded);
  return new TextDecoder().decode(Uint8Array.from(binary, (c) => c.charCodeAt(0)));
}

export function encodeElevationState(state: ElevatedUiState): string {
  return toBase64Url(JSON.stringify(state));
}

/** 解析失败或字段越界时只恢复能识别的部分，绝不因一个坏参数阻止启动。 */
export function decodeElevationState(raw: string | null | undefined): Partial<ElevatedUiState> | null {
  if (!raw) return null;
  try {
    const parsed = JSON.parse(fromBase64Url(raw)) as Partial<ElevatedUiState>;
    const out: Partial<ElevatedUiState> = {};
    if (parsed.view === "process" || parsed.view === "port") out.view = parsed.view;
    if (typeof parsed.search === "string") out.search = parsed.search.slice(0, 200);
    if (PROTOCOLS.includes(parsed.protocol as ProtocolFilter)) out.protocol = parsed.protocol as ProtocolFilter;
    if (STATUSES.includes(parsed.status as StatusFilter)) out.status = parsed.status as StatusFilter;
    if (FAMILIES.includes(parsed.family as FamilyFilter)) out.family = parsed.family as FamilyFilter;
    if (typeof parsed.listeningOnly === "boolean") out.listeningOnly = parsed.listeningOnly;
    return out;
  } catch {
    return null;
  }
}
