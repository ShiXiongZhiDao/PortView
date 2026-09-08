import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { pinyin } from "pinyin-pro";
import type { ConnectionInfo } from "../types";

/** 带预计算搜索索引的行 */
interface Row extends ConnectionInfo {
  _searchIndex: string;
}

export type ProtocolFilter = "全部" | "TCP" | "UDP";
export type StatusFilter = "全部" | "LISTENING" | "ESTABLISHED" | "TIME_WAIT" | "CLOSE_WAIT" | "其他";

/** 已知状态集合，"其他"即不属于这些状态的条目（含 UDP 的 "-" 与 SYN_SENT 等） */
const KNOWN_STATUSES = ["LISTENING", "ESTABLISHED", "TIME_WAIT", "CLOSE_WAIT"];

/** 构建单行模糊搜索索引：端口/进程名/PID/路径/软件名/拼音（全拼+首字母） */
function buildSearchIndex(r: ConnectionInfo): string {
  const text = `${r.processName} ${r.softwareName} ${r.processPath}`;
  const full = pinyin(text, { toneType: "none", type: "array" }).join("");
  const initial = pinyin(text, { pattern: "first", toneType: "none", type: "array" }).join("");
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
    full,
    initial,
  ]
    .join(" ")
    .toLowerCase();
}

export const useConnectionsStore = defineStore("connections", () => {
  const rows = ref<Row[]>([]);
  const loading = ref(false);
  const error = ref("");
  const lastUpdated = ref<number | null>(null);

  // 筛选状态
  const search = ref("");
  const protocol = ref<ProtocolFilter>("全部");
  const status = ref<StatusFilter>("全部");

  async function refresh() {
    loading.value = true;
    try {
      const data = await invoke<ConnectionInfo[]>("get_connections");
      rows.value = data.map((r) => ({ ...r, _searchIndex: buildSearchIndex(r) }));
      lastUpdated.value = Date.now();
      error.value = "";
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  /** 过滤后的行（搜索 + 协议 + 状态叠加） */
  const filteredRows = computed(() => {
    const q = search.value.trim().toLowerCase();
    return rows.value.filter((r) => {
      if (protocol.value !== "全部" && r.protocol !== protocol.value) return false;
      if (status.value !== "全部") {
        const ok =
          status.value === "其他"
            ? !KNOWN_STATUSES.includes(r.state)
            : r.state === status.value;
        if (!ok) return false;
      }
      if (q && !r._searchIndex.includes(q)) return false;
      return true;
    });
  });

  /** 统计口径（基于全部数据）：TCP 总数 / UDP 总数 / 监听端口 / 去重进程数 */
  const stats = computed(() => {
    let tcp = 0;
    let udp = 0;
    let listening = 0;
    const pids = new Set<number>();
    for (const r of rows.value) {
      if (r.protocol === "TCP") tcp++;
      else if (r.protocol === "UDP") udp++;
      if (r.state === "LISTENING") listening++;
      pids.add(r.pid);
    }
    return { tcp, udp, listening, procs: pids.size };
  });

  /** 结束进程（taskkill /F），返回结果或抛出错误 */
  async function kill(pid: number): Promise<string> {
    return await invoke<string>("kill_process", { pid });
  }

  return {
    rows,
    loading,
    error,
    lastUpdated,
    search,
    protocol,
    status,
    filteredRows,
    stats,
    refresh,
    kill,
  };
});
