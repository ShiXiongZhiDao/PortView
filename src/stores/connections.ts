import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { pinyin } from "pinyin-pro";
import type { ConnectionInfo, ProcessInfo, SystemSnapshot } from "../types";

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
export type FamilyFilter = "全部" | "IPv4" | "IPv6";

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
    r.addressFamily,
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

/** 构建进程行模糊搜索索引：进程名/PID/路径/软件名/拼音/端口/连接地址 */
function buildProcessSearchIndex(p: ProcessInfo, conns: ConnectionInfo[]): string {
  const text = `${p.name} ${p.software} ${p.path}`;
  return [
    p.name,
    p.path,
    p.software,
    String(p.pid),
    pinyinIndex(text),
    ...conns.flatMap((c) => [
      String(c.localPort),
      String(c.remotePort),
      c.localAddress,
      c.remoteAddress,
    ]),
  ]
    .join(" ")
    .toLowerCase();
}

export const useConnectionsStore = defineStore("connections", () => {
  // 连接数据（来自最近一次成功快照）
  const rows = ref<Row[]>([]);
  // 进程数据（含聚合端口）
  const processes = ref<ProcessRow[]>([]);
  // 快照元信息
  const capturedAtMs = ref<number | null>(null);
  const warnings = ref<SystemSnapshot["warnings"]>([]);
  const isElevated = ref(false);
  const durationMs = ref(0);

  // 刷新状态：inFlight 锁防止重入（定时刷新与手动刷新共用）
  const loading = ref(false);
  const error = ref("");

  // 筛选状态
  const search = ref("");
  const protocol = ref<ProtocolFilter>("全部");
  const status = ref<StatusFilter>("全部");
  const family = ref<FamilyFilter>("全部");
  /** 仅监听：等价于 TCP + LISTENING，不含 UDP */
  const listeningOnly = ref(false);
  /** 当前视图：进程 / 端口 */
  const view = ref<"process" | "port">("process");

  /** 统一快照：一次 IPC 取回进程 + 连接 + 告警（见 ADR-0009）。
   *  刷新失败时保留上次成功数据，只更新 error，不清空列表。 */
  async function refreshSnapshot(): Promise<void> {
    if (loading.value) return;
    loading.value = true;
    try {
      const snap = await invoke<SystemSnapshot>("get_system_snapshot");
      rows.value = snap.connections.map((r) => ({ ...r, _searchIndex: buildSearchIndex(r) }));
      const infoByPid = new Map<number, { ports: Set<number>; states: Set<string> }>();
      for (const r of snap.connections) {
        const info = infoByPid.get(r.pid) ?? { ports: new Set<number>(), states: new Set<string>() };
        info.ports.add(r.localPort);
        info.states.add(r.state);
        infoByPid.set(r.pid, info);
      }
      const connsByPid = new Map<number, ConnectionInfo[]>();
      for (const r of snap.connections) {
        const arr = connsByPid.get(r.pid) ?? [];
        arr.push(r);
        connsByPid.set(r.pid, arr);
      }
      processes.value = snap.processes.map((p) => {
        const info = infoByPid.get(p.pid);
        const ports = info ? Array.from(info.ports).sort((a, b) => a - b) : [];
        return {
          ...p,
          _ports: ports,
          _states: info?.states ?? new Set<string>(),
          _searchIndex: buildProcessSearchIndex(p, connsByPid.get(p.pid) ?? []),
        };
      });
      capturedAtMs.value = snap.capturedAtMs;
      warnings.value = snap.warnings;
      isElevated.value = snap.isElevated;
      durationMs.value = snap.durationMs;
      error.value = "";
    } catch (e) {
      // 保留上次成功快照，只提示错误
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 兼容旧调用点：刷新 = 取一次统一快照 */
  async function refresh(): Promise<void> {
    await refreshSnapshot();
  }

  /** 是否有连接级筛选条件（含搜索/协议/状态/地址族/仅监听）。
   *  为真时进程视图只保留拥有匹配连接的进程；为假时保留无网络进程。 */
  const hasConnectionFilter = computed(
    () =>
      search.value.trim() !== "" ||
      protocol.value !== "全部" ||
      status.value !== "全部" ||
      family.value !== "全部" ||
      listeningOnly.value,
  );

  /** 单条连接是否匹配当前协议/状态/地址族/仅监听（不含搜索） */
  function matchConnection(r: Row): boolean {
    // "仅监听"等价于 TCP + LISTENING，明确排除 UDP。
    // 协议/状态下拉此时被禁用，但地址族仍可叠加，否则「IPv6 + 仅监听」会列出 IPv4。
    if (listeningOnly.value) {
      if (r.protocol !== "TCP" || r.state !== "LISTENING") return false;
      return family.value === "全部" || r.addressFamily === family.value;
    }
    if (protocol.value !== "全部" && r.protocol !== protocol.value) return false;
    if (family.value !== "全部" && r.addressFamily !== family.value) return false;
    if (status.value !== "全部") {
      const ok =
        status.value === "其他"
          ? !KNOWN_STATUSES.includes(r.state)
          : r.state === status.value;
      if (!ok) return false;
    }
    return true;
  }

  /** 过滤后的连接行（端口视图直接展示）：搜索 + 协议 + 状态 + 地址族 + 仅监听 */
  const filteredConnections = computed<Row[]>(() => {
    const q = search.value.trim().toLowerCase();
    return rows.value.filter((r) => matchConnection(r) && (!q || r._searchIndex.includes(q)));
  });

  /** 进程统计与展开明细的唯一事实源：直接用筛选后的连接（含搜索），
   *  保证表头数字、端口摘要、展开明细三者始终同源（设计包 §3.6）。 */
  const matchedByPid = computed<Map<number, Row[]>>(() => {
    const map = new Map<number, Row[]>();
    for (const r of filteredConnections.value) {
      const arr = map.get(r.pid) ?? [];
      arr.push(r);
      map.set(r.pid, arr);
    }
    return map;
  });

  /** 过滤后的进程行：进程统计与展开明细都基于同一组匹配连接。
   *  注意：搜索词既能匹配进程自身（含拼音），也能匹配其连接地址/端口；
   *  但无网络进程在没有任何连接筛选条件时必须保留。 */
  const filteredProcesses = computed<ProcessRow[]>(() => {
    const q = search.value.trim().toLowerCase();
    const out: ProcessRow[] = [];
    for (const p of processes.value) {
      if (q && !p._searchIndex.includes(q)) continue;
      const matchedConns = matchedByPid.value.get(p.pid) ?? [];
      // 有连接筛选条件时，只保留拥有至少一条匹配连接的进程；
      // 例外：无网络进程按自身名称/拼音命中搜索时仍保留
      if (hasConnectionFilter.value && matchedConns.length === 0) {
        const selfHit = q && p._searchIndex.includes(q);
        if (!selfHit) continue;
      }
      if (!hasConnectionFilter.value) {
        out.push(p);
        continue;
      }
      // 统计与端口摘要统一按匹配连接重算，避免表头数字与展开明细打架
      const ports = Array.from(new Set(matchedConns.map((c) => c.localPort))).sort((a, b) => a - b);
      const counts = filteredCountsOf(p.pid);
      out.push({
        ...p,
        _ports: ports,
        _states: new Set(matchedConns.map((c) => c.state)),
        tcp: counts.tcp,
        udp: counts.udp,
        listening: counts.listening,
      });
    }
    return out;
  });

  /** 某进程在当前筛选下的连接明细（展开行用，与进程统计同源） */
  function connectionsOf(pid: number): Row[] {
    return filteredConnections.value.filter((r) => r.pid === pid);
  }

  /** 进程行在"当前筛选结果"下的连接计数（展开/统计口径一致） */
  function filteredCountsOf(pid: number): { tcp: number; udp: number; listening: number } {
    const conns = matchedByPid.value.get(pid) ?? [];
    let tcp = 0;
    let udp = 0;
    let listening = 0;
    for (const c of conns) {
      if (c.protocol === "TCP") {
        tcp++;
        if (c.state === "LISTENING") listening++;
      } else {
        udp++;
      }
    }
    return { tcp, udp, listening };
  }

  /** 统计口径：全部进程 + 连接聚合（不受筛选影响，反映快照整体） */
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

  /** 结束进程（带进程实例身份校验）；失败时抛出结构化错误码，由调用方本地化 */
  async function kill(pid: number, creationTime100ns: string | null): Promise<void> {
    await invoke("kill_process", {
      pid,
      expectedCreationTime100ns: creationTime100ns,
    });
  }

  return {
    rows,
    processes,
    capturedAtMs,
    warnings,
    isElevated,
    durationMs,
    loading,
    error,
    search,
    protocol,
    status,
    family,
    listeningOnly,
    view,
    hasConnectionFilter,
    filteredConnections,
    filteredProcesses,
    connectionsOf,
    filteredCountsOf,
    stats,
    refresh,
    refreshSnapshot,
    kill,
  };
});
