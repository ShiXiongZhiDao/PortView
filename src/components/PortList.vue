<script setup lang="ts">
import { computed, reactive, ref } from "vue";
import { useConnectionsStore } from "../stores/connections";
import { useI18n } from "../i18n";
import { useClipboard } from "../composables/useClipboard";
import type { ConnectionInfo } from "../types";

const store = useConnectionsStore();
const { t } = useI18n();
const { copy } = useClipboard();

type SortKey = "localPort" | "remoteAddr";
const sortKey = ref<SortKey>("localPort");
const sortDir = ref<"asc" | "desc">("asc");
function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortDir.value = "asc";
  }
}
/** 端口默认升序；远程地址按地址排序，端口作为同一地址下的次级键（设计包 §2） */
const rows = computed<ConnectionInfo[]>(() => {
  const list = [...store.filteredConnections];
  const dir = sortDir.value === "asc" ? 1 : -1;
  return list.sort((a, b) => {
    if (sortKey.value === "localPort") return dir * (a.localPort - b.localPort);
    const byAddress = a.remoteAddress.localeCompare(b.remoteAddress, undefined, {
      numeric: true,
      sensitivity: "base",
    });
    return dir * (byAddress || a.remotePort - b.remotePort);
  });
});

function stateColor(state: string): string {
  if (state === "LISTENING") return "var(--pv-green-text)";
  if (state === "ESTABLISHED") return "var(--pv-blue-text)";
  return "var(--pv-text-faint)";
}
function stateBg(protocol: string): string {
  return protocol === "TCP" ? "var(--pv-green-bg)" : "var(--pv-blue-bg)";
}
function stateFg(protocol: string): string {
  return protocol === "TCP" ? "var(--pv-green-text)" : "var(--pv-blue-text)";
}

/** 端口贴纸：按连接状态着色，读不到状态时退回中性色（暗色主题自动适配） */
function portChipStyle(c: ConnectionInfo): Record<string, string> {
  if (c.protocol === "UDP") {
    return {
      background: "var(--pv-violet-bg)",
      color: "var(--pv-violet-text)",
      borderColor: "var(--pv-violet-text)",
    };
  }
  if (c.state === "LISTENING") {
    return {
      background: "var(--pv-green-bg)",
      color: "var(--pv-green-text)",
      borderColor: "var(--pv-green-text)",
    };
  }
  if (c.state === "ESTABLISHED") {
    return {
      background: "var(--pv-blue-bg)",
      color: "var(--pv-blue-text)",
      borderColor: "var(--pv-blue-text)",
    };
  }
  if (c.state === "TIME_WAIT") {
    return {
      background: "var(--pv-amber-bg)",
      color: "var(--pv-amber-text)",
      borderColor: "var(--pv-amber-text)",
    };
  }
  return {
    background: "var(--pv-card-2)",
    color: "var(--pv-text-soft)",
    borderColor: "var(--pv-border)",
  };
}

/* ---------- 列宽：与进程列表一致的拖拽 + 持久化 ---------- */
type ColKey = "localPort" | "protocol" | "state" | "localAddr" | "remoteAddr" | "process" | "actions";

/** 默认分配：地址列不再无限胀，进程/操作列留够，整表约 1170px（1280 窗口内容区约 1224px） */
const COL_DEFAULTS: Record<ColKey, { width: number; min: number }> = {
  localPort: { width: 88, min: 64 },
  protocol: { width: 64, min: 52 },
  state: { width: 116, min: 84 },
  localAddr: { width: 244, min: 140 },
  remoteAddr: { width: 244, min: 140 },
  process: { width: 210, min: 140 },
  actions: { width: 108, min: 96 },
};
const WIDTH_KEY = "port-view-port-col-widths";
const colKeys = Object.keys(COL_DEFAULTS) as ColKey[];

function loadColWidths(): Partial<Record<ColKey, number>> {
  try {
    const raw = JSON.parse(localStorage.getItem(WIDTH_KEY) || "{}") as Record<string, unknown>;
    const out: Partial<Record<ColKey, number>> = {};
    (Object.keys(COL_DEFAULTS) as ColKey[]).forEach((k) => {
      const v = Number(raw[k]);
      if (Number.isFinite(v) && v > 0) out[k] = Math.min(Math.max(v, COL_DEFAULTS[k].min), 600);
    });
    return out;
  } catch {
    return {};
  }
}

const colWidths = reactive<Record<ColKey, number>>({
  ...(Object.fromEntries(
    (Object.keys(COL_DEFAULTS) as ColKey[]).map((k) => [k, COL_DEFAULTS[k].width]),
  ) as Record<ColKey, number>),
  ...loadColWidths(),
});
function widthOf(key: ColKey): string {
  return `${colWidths[key]}px`;
}
const CELL_GAP = 12;
const H_PADDING = 40;
const totalWidth = computed(() =>
  (Object.keys(COL_DEFAULTS) as ColKey[]).reduce((sum, k) => sum + colWidths[k], 0),
);
const innerWidth = computed(() => totalWidth.value + (colKeys.length - 1) * CELL_GAP + H_PADDING);

let resizing: { key: ColKey; startX: number; startWidth: number } | null = null;
function startResize(e: MouseEvent, key: ColKey) {
  resizing = { key, startX: e.clientX, startWidth: colWidths[key] };
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  document.addEventListener("mousemove", onResizeMove);
  document.addEventListener("mouseup", stopResize, { once: true });
}
function onResizeMove(e: MouseEvent) {
  if (!resizing) return;
  const dx = e.clientX - resizing.startX;
  colWidths[resizing.key] = Math.min(
    Math.max(resizing.startWidth + dx, COL_DEFAULTS[resizing.key].min),
    600,
  );
}
function stopResize() {
  resizing = null;
  document.body.style.cursor = "";
  document.body.style.userSelect = "";
  document.removeEventListener("mousemove", onResizeMove);
  try {
    localStorage.setItem(WIDTH_KEY, JSON.stringify(colWidths));
  } catch {
    /* 持久化失败忽略 */
  }
}

/* 表头与列表横向滚动同步 */
const headerScrollEl = ref<HTMLElement | null>(null);
const listScrollEl = ref<HTMLElement | null>(null);
function onListScroll() {
  const el = listScrollEl.value;
  if (headerScrollEl.value && el) headerScrollEl.value.scrollLeft = el.scrollLeft;
}
</script>

<template>
  <div class="pv-card flex h-full flex-col overflow-hidden">
    <!-- 表头：贴纸精修——实线分隔、粗底边，列宽可拖拽 -->
    <div ref="headerScrollEl" class="header-scroll shrink-0 overflow-x-auto">
      <div
        class="flex gap-3 px-5 py-3 text-xs font-extrabold tracking-wider"
        :style="{
          background: 'var(--pv-card-2)',
          borderBottom: '3px solid var(--pv-stroke)',
          width: innerWidth + 'px',
          minWidth: '100%',
        }"
      >
        <div
          v-for="key in colKeys"
          :key="key"
          class="relative shrink-0 select-none"
          :style="{ width: widthOf(key) }"
        >
          <button
            v-if="key === 'localPort' || key === 'remoteAddr'"
            class="sort-head"
            :style="{ color: sortKey === (key === 'localPort' ? 'localPort' : 'remoteAddr') ? 'var(--pv-green-text)' : 'inherit' }"
            @click="toggleSort(key === 'localPort' ? 'localPort' : 'remoteAddr')"
          >
            {{ key === 'localPort' ? t('port.col.local') : t('port.col.remoteAddr') }}
            {{ sortKey === (key === 'localPort' ? 'localPort' : 'remoteAddr') ? (sortDir === 'asc' ? '▲' : '▼') : '↕' }}
          </button>
          <template v-else>
            {{
              key === 'protocol' ? t('port.col.protocol')
              : key === 'state' ? t('port.col.state')
              : key === 'localAddr' ? t('port.col.localAddr')
              : key === 'process' ? t('port.col.process')
              : t('port.col.actions')
            }}
          </template>
          <span class="col-resizer" @mousedown.prevent.stop="startResize($event, key)"></span>
        </div>
      </div>
    </div>

    <!-- 列表 -->
    <div ref="listScrollEl" class="min-h-0 flex-1 overflow-auto" @scroll="onListScroll">
      <div v-if="rows.length" :style="{ width: innerWidth + 'px', minWidth: '100%' }">
        <div
          v-for="(r, i) in rows"
          :key="`${r.protocol}-${r.pid}-${r.localPort}-${i}`"
          class="port-row flex min-h-[46px] items-center gap-3 px-5 py-2 text-xs"
          :style="{ borderBottom: '1px solid var(--pv-border)' }"
          :title="t('port.dblclickTip')"
          @dblclick="store.view = 'process'"
        >
          <button
            class="pv-text-btn port-chip shrink-0"
            :style="{ width: widthOf('localPort'), ...portChipStyle(r) }"
            :title="t('port.copyPort')"
            @click="copy(String(r.localPort), t('port.col.local'))"
          >
            {{ r.localPort }}
          </button>

          <span class="shrink-0" :style="{ width: widthOf('protocol') }">
            <span
              class="rounded-full px-2 py-px text-[10px] font-extrabold"
              :style="{ background: stateBg(r.protocol), color: stateFg(r.protocol) }"
            >
              {{ r.protocol }}
            </span>
          </span>

          <span class="shrink-0 text-[10px] font-bold" :style="{ width: widthOf('state'), color: stateColor(r.state) }">
            {{ r.state === "-" ? "—" : r.state }}
          </span>

          <button
            class="pv-text-btn min-w-0 shrink-0 truncate text-left tabular-nums"
            :style="{ width: widthOf('localAddr'), color: 'var(--pv-text-soft)' }"
            :title="t('port.copyLocal')"
            @click="copy(r.localAddress, t('port.col.localAddr'))"
          >
            {{ r.localAddress }}
          </button>

          <button
            v-if="r.remoteAddress !== '-'"
            class="pv-text-btn min-w-0 shrink-0 truncate text-left tabular-nums"
            :style="{ width: widthOf('remoteAddr'), color: 'var(--pv-text-soft)' }"
            :title="t('port.copyRemote')"
            @click="copy(r.remoteAddress, t('port.col.remoteAddr'))"
          >
            {{ r.remoteAddress }}
          </button>
          <span v-else class="min-w-0 shrink-0 truncate" :style="{ width: widthOf('remoteAddr'), color: 'var(--pv-text-faint)' }">—</span>

          <!-- 进程名与 PID 内嵌在一格，两者都可点击复制 -->
          <span class="flex min-w-0 shrink-0 flex-col" :style="{ width: widthOf('process') }">
            <button
              class="pv-text-btn truncate text-left font-bold"
              :style="{ color: 'var(--pv-text)' }"
              :title="t('port.copyProcess')"
              @click="copy(r.processName || `pid${r.pid}`, t('port.col.process'))"
            >
              {{ r.processName || `pid${r.pid}` }}
            </button>
            <button
              class="pv-text-btn shrink-0 text-left text-[10px] tabular-nums"
              :style="{ color: 'var(--pv-text-faint)' }"
              :title="t('port.copyPid')"
              @click="copy(String(r.pid), 'PID')"
            >
              PID {{ r.pid }}
            </button>
          </span>

          <span class="shrink-0" :style="{ width: widthOf('actions') }">
            <button class="pv-mini-btn row-action-btn" @click="store.view = 'process'">
              {{ t("port.viewProcess") }}
            </button>
          </span>
        </div>
      </div>

      <!-- 空状态：区分无结果 / 采集失败 / 部分 warning -->
      <div v-else class="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
        <div class="text-sm font-bold" :style="{ color: 'var(--pv-text-soft)' }">
          {{ store.error ? t("empty.collectFailed") : t("empty.noResult") }}
        </div>
        <div v-if="store.error" class="max-w-md text-xs" :style="{ color: '#DC2626' }">{{ store.error }}</div>
        <div v-else class="text-xs" :style="{ color: 'var(--pv-text-faint)' }">{{ t("empty.tip") }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.header-scroll {
  scrollbar-width: none;
}
.header-scroll::-webkit-scrollbar {
  display: none;
}
.sort-head {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 0;
  border: 0;
  background: transparent;
  font: inherit;
  color: inherit;
  letter-spacing: inherit;
  cursor: pointer;
}
.sort-head:hover {
  color: var(--pv-green-text);
}
.col-resizer {
  position: absolute;
  top: 0;
  right: -4px;
  width: 8px;
  height: 100%;
  cursor: col-resize;
  z-index: 5;
  border-radius: 4px;
}
.col-resizer:hover,
.col-resizer:active {
  background: rgba(34, 197, 94, 0.28);
}
.port-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 28px;
  padding: 3px 8px;
  border: 2px solid;
  border-radius: 8px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.port-chip:hover {
  text-decoration: none;
  filter: brightness(0.97);
}
.port-row:hover {
  background: var(--pv-row-hover);
  box-shadow: inset 3px 0 0 var(--pv-green);
}
</style>
