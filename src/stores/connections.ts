import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { pinyin } from "pinyin-pro";
import type { ConnectionInfo, ProcessInfo } from "../types";

/** 带预计算搜索索引的连接行 */
interface Row extends ConnectionInfo {
  _searchIndex: string;
}

/** 带预计算搜索索引的进程行（含聚合端口列表与状态集合） */
interface ProcessRow extends ProcessInfo {
  _ports: number[];
  _states: Set<string>;
  _searchIndex: string;
}

export type ProtocolFilter = "全部" | "TCP" | "UDP";
export type StatusFilter = "全部" | "LISTENING" | "ESTABLISHED" | "TIME_WAIT" | "CLOSE_WAIT" | "其他";

/** 已知状态集合，"其他"即不属于这些状态的条目（含 UDP 的 "-" 与 SYN_SENT 等） */
const KNOWN_STATUSES = ["LISTENING", "ESTABLISHED", "TIME_WAIT", "CLOSE_WAIT"];

/** 文本 → 拼音索引（全拼 + 首字母） */
function pinyinIndex(text: string): string {
  const full = pinyin(text, { toneType: "none", type: "array" }).join("");
  const initial = pinyin(text, { pattern: "first", toneType: "none", type: "array" }).join("");
  return `${full} ${initial}`;
}

/** 构建连接行模糊搜索索引：端口/进程名/PID/路径/软件名/拼音 */
function buildSearchIndex(r: ConnectionInfo): string {
  const text = `${r.processName} ${r.softwareName} ${r.processPath}`;
  return [
    r.protocol,
    String(r.localPort),
    String(r.remotePort),
    r.localAddress,
    r.remoteAddress,
    r.state,
    r.processName,
    r.processPath,
    r.softwareName,
    String(r.pid),
    pinyinIndex(text),
  ]
    .join(" ")
    .toLowerCase();
}

/** 构建进程行模糊搜索索引：进程名/PID/路径/软件名/拼音/端口 */
function buildProcessSearchIndex(p: ProcessInfo, ports: number[]): string {
  const text = `${p.name} ${p.software} ${p.path}`;
  return [
    p.name,
    p.path,
    p.software,
    String(p.pid),
    pinyinIndex(text),
    ...ports.map(String),
  ]
    .join(" ")
    .toLowerCase();
}

export const useConnectionsStore = defineStore("connections", () => {
  // 连接数据
  const rows = ref<Row[]>([]);
  // 进程数据（含聚合端口）
  const processes = ref<ProcessRow[]>([]);

  const loading = ref(false);
  const error = ref("");
  const lastUpdated = ref<number | null>(null);

  // 筛选状态
  const search = ref("");
  const protocol = ref<ProtocolFilter>("全部");
  const status = ref<StatusFilter>("全部");

  /** 合并视图数据源：进程列表 + 连接明细一次取回 */
  async function fetchAll() {
    loading.value = true;
    try {
      const [connData, procData] = await Promise.all([
        invoke<ConnectionInfo[]>("get_connections"),
        invoke<ProcessInfo[]>("get_processes"),
      ]);
      rows.value = connData.map((r) => ({ ...r, _searchIndex: buildSearchIndex(r) }));
      const infoByPid = new Map<number, { ports: Set<number>; states: Set<string> }>();
      for (const r of connData) {
        const info = infoByPid.get(r.pid) ?? { ports: new Set<number>(), states: new Set<string>() };
        info.ports.add(r.localPort);
        info.states.add(r.state);
        infoByPid.set(r.pid, info);
      }
      processes.value = procData.map((p) => {
        const info = infoByPid.get(p.pid);
        const ports = info ? Array.from(info.ports).sort((a, b) => a - b) : [];
        return {
          ...p,
          _ports: ports,
          _states: info?.states ?? new Set<string>(),
          _searchIndex: buildProcessSearchIndex(p, ports),
        };
      });
      lastUpdated.value = Date.now();
      error.value = "";
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 手动刷新（进程 + 连接一并刷新） */
  async function refresh() {
    await fetchAll();
  }

  /** 过滤后的进程行：搜索 + 协议 + 状态叠加（协议/状态映射到进程维度） */
  const filteredProcesses = computed(() => {
    const q = search.value.trim().toLowerCase();
    return processes.value.filter((p) => {
      if (protocol.value === "TCP" && p.tcp === 0) return false;
      if (protocol.value === "UDP" && p.udp === 0) return false;
      if (status.value !== "全部") {
        const ok =
          status.value === "其他"
            ? [...p._states].some((s) => !KNOWN_STATUSES.includes(s))
            : p._states.has(status.value);
        if (!ok) return false;
      }
      if (q && !p._searchIndex.includes(q)) return false;
      return true;
    });
  });

  /** 某进程的连接明细（展开行用，不随搜索过滤） */
  function connectionsOf(pid: number): Row[] {
    return rows.value.filter((r) => r.pid === pid);
  }

  /** 统计口径：全部进程 + 连接聚合 */
  const stats = computed(() => {
    let tcp = 0;
    let udp = 0;
    let listening = 0;
    for (const r of rows.value) {
      if (r.protocol === "TCP") tcp++;
      else if (r.protocol === "UDP") udp++;
      if (r.state === "LISTENING") listening++;
    }
    return { tcp, udp, listening, procs: processes.value.length };
  });

  /** 结束进程（taskkill /F），返回结果或抛出错误 */
  async function kill(pid: number): Promise<string> {
    return await invoke<string>("kill_process", { pid });
  }

  return {
    rows,
    processes,
    loading,
    error,
    lastUpdated,
    search,
    protocol,
    status,
    filteredProcesses,
    connectionsOf,
    stats,
    refresh,
    kill,
  };
});
