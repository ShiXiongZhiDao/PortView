/**
 * 定时快照刷新（ADR-0007）——模块级单例，全应用只有这一个 timer。
 *
 * 默认开、间隔 3 秒；"隐藏/失焦暂停"按 ADR-0007 明确不实现。
 * inFlight 由 store 的 loading 锁保证：慢请求期间跳过的 tick 不排队叠加。
 * 开关/间隔状态持久化；刷新不改动搜索、筛选、排序、列宽与滚动位置。
 */
import { ref, watch } from "vue";
import type { useConnectionsStore } from "../stores/connections";

type Store = ReturnType<typeof useConnectionsStore>;

const ENABLED_KEY = "port-view-auto-refresh";
const INTERVAL_KEY = "port-view-refresh-interval";

export const REFRESH_INTERVALS = [1, 3, 5, 10] as const;
export type RefreshInterval = (typeof REFRESH_INTERVALS)[number];

function loadEnabled(): boolean {
  try {
    const raw = localStorage.getItem(ENABLED_KEY);
    return raw === null ? true : raw === "1"; // 默认开（ADR-0007）
  } catch {
    return true;
  }
}

function loadInterval(): RefreshInterval {
  try {
    const n = Number(localStorage.getItem(INTERVAL_KEY));
    return REFRESH_INTERVALS.includes(n as RefreshInterval) ? (n as RefreshInterval) : 3;
  } catch {
    return 3;
  }
}

const enabled = ref<boolean>(loadEnabled());
const intervalSec = ref<RefreshInterval>(loadInterval());

let timer: ReturnType<typeof setInterval> | null = null;
let started = false;

function clear() {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
}

function restart(store: Store) {
  clear();
  if (!enabled.value) return;
  timer = setInterval(() => {
    // store.refreshSnapshot 内部有 loading 锁：inFlight 时直接返回，不排队。
    void store.refreshSnapshot();
  }, intervalSec.value * 1000);
}

function setEnabled(store: Store, v: boolean) {
  enabled.value = v;
  try {
    localStorage.setItem(ENABLED_KEY, v ? "1" : "0");
  } catch {
    /* 存储不可用只影响持久化 */
  }
  restart(store);
}

function setIntervalSec(store: Store, v: RefreshInterval) {
  intervalSec.value = v;
  try {
    localStorage.setItem(INTERVAL_KEY, String(v));
  } catch {
    /* 同上 */
  }
  restart(store);
}

/** 单例入口：传 store 仅供首次启动时绑定 timer；之后各处调用拿到同一份状态。 */
export function useAutoRefresh(store: Store) {
  if (!started) {
    started = true;
    watch([enabled, intervalSec], () => restart(store));
    restart(store);
  }
  return {
    enabled,
    intervalSec,
    setEnabled: (v: boolean) => setEnabled(store, v),
    setIntervalSec: (v: RefreshInterval) => setIntervalSec(store, v),
    stop: clear,
  };
}
