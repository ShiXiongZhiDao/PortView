<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDialog, useMessage } from "naive-ui";
import { useConnectionsStore } from "../stores/connections";
import { useTheme } from "../composables/useTheme";
import { useI18n } from "../i18n";
import { useClipboard } from "../composables/useClipboard";
import { useElevation } from "../composables/useElevation";
import { errorCode, killErrorMessage } from "../lib/kill-error";
import { isCriticalProcess } from "../lib/process-safety";
import ProcessDetail from "./ProcessDetail.vue";
import type { MetricStatus, ProcessInfo } from "../types";

const store = useConnectionsStore();
const message = useMessage();
const dialog = useDialog();
const { isDark } = useTheme();
const { t } = useI18n();
const { copy } = useClipboard();
const { elevate } = useElevation();

/** 详情抽屉：单击进程名复制，双击整行打开详情 */
const detailPid = ref<number | null>(null);
const showDetail = ref(false);
function openDetail(pid: number) {
  detailPid.value = pid;
  showDetail.value = true;
}
function onRowDblClick(e: MouseEvent, pid: number) {
  if ((e.target as HTMLElement).closest(".row-actions")) return;
  openDetail(pid);
}

/** 当前展开的进程 PID（同时只展开一个） */
const expandedPid = ref<number | null>(null);
function toggle(pid: number) {
  expandedPid.value = expandedPid.value === pid ? null : pid;
}

/** 可排序字段与方向：默认按 CPU 从大到小 */
type SortKey = "memory" | "cpu";
const sortKey = ref<SortKey>("cpu");
const sortDir = ref<"desc" | "asc">("desc");
function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === "desc" ? "asc" : "desc";
  } else {
    sortKey.value = key;
    sortDir.value = "desc";
  }
}
function metricValue(p: ProcessInfo, key: SortKey): number {
  const v = key === "cpu" ? p.cpuPercent : p.memoryBytes;
  return v ?? -1;
}
const rows = computed<(ProcessInfo & { _ports: number[] })[]>(() => {
  const list = [...(store.filteredProcesses as (ProcessInfo & { _ports: number[] })[])];
  const key = sortKey.value;
  const dir = sortDir.value === "desc" ? -1 : 1;
  return list.sort((a, b) => dir * (metricValue(a, key) - metricValue(b, key)));
});

/* ---------- 列定义与可拖拽列宽（贴纸精修：进程 / CPU / 内存 / 连接 / 端口 / 操作） ---------- */
type ColKey = "process" | "cpu" | "memory" | "connections" | "ports" | "actions";

/** 默认宽度与端口视图保持同一节奏：整表约 1160px，1280 窗口内容区约 1224px */
const COL_DEFAULTS: Record<ColKey, { width: number; min: number }> = {
  process: { width: 300, min: 180 },
  cpu: { width: 110, min: 80 },
  memory: { width: 110, min: 76 },
  connections: { width: 140, min: 104 },
  ports: { width: 280, min: 140 },
  actions: { width: 132, min: 120 },
};
const WIDTH_KEY = "port-view-col-widths-v2";

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
const totalWidth = computed(() =>
  (Object.keys(COL_DEFAULTS) as ColKey[]).reduce((sum, k) => sum + colWidths[k], 0),
);
const columns = computed<{ key: ColKey; label: string; sort?: SortKey }[]>(() => [
  { key: "process", label: t("list.col.process") },
  { key: "cpu", label: t("list.col.cpu"), sort: "cpu" },
  { key: "memory", label: t("list.col.memory"), sort: "memory" },
  { key: "connections", label: t("list.col.connections") },
  { key: "ports", label: t("list.col.ports") },
  { key: "actions", label: t("list.col.actions") },
]);

/** 表头/行共用布局参数（与 class gap-3、px-5 保持一致） */
const CELL_GAP = 12;
const H_PADDING = 40;
const innerWidth = computed(() => totalWidth.value + (columns.value.length - 1) * CELL_GAP + H_PADDING);

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

/* ---------- 表头与列表横向滚动同步 ---------- */
const headerScrollEl = ref<HTMLElement | null>(null);
const listScrollEl = ref<HTMLElement | null>(null);
function onListScroll() {
  const el = listScrollEl.value;
  if (headerScrollEl.value && el) headerScrollEl.value.scrollLeft = el.scrollLeft;
  closeCtx();
}

/* ---------- 右键菜单：打开程序地址 / 结束进程 ---------- */
interface CtxState {
  x: number;
  y: number;
  row: ProcessInfo & { _ports: number[] };
}
const ctxMenu = ref<CtxState | null>(null);
function openCtx(e: MouseEvent, row: ProcessInfo & { _ports: number[] }) {
  const menuW = 200;
  const menuH = 92;
  ctxMenu.value = {
    x: Math.min(e.clientX, window.innerWidth - menuW - 8),
    y: Math.min(e.clientY, window.innerHeight - menuH - 8),
    row,
  };
}
function closeCtx() {
  ctxMenu.value = null;
}
function onDocClick() {
  closeCtx();
}
function onDocKey(e: KeyboardEvent) {
  if (e.key === "Escape") closeCtx();
}
watch(ctxMenu, (v) => {
  if (v) {
    document.addEventListener("click", onDocClick);
    document.addEventListener("keydown", onDocKey);
  } else {
    document.removeEventListener("click", onDocClick);
    document.removeEventListener("keydown", onDocKey);
  }
});

async function ctxOpenLocation() {
  const row = ctxMenu.value?.row;
  closeCtx();
  if (!row) return;
  if (!row.path) {
    message.warning(t("ctx.noPath"));
    return;
  }
  try {
    await invoke("open_in_explorer", { path: row.path });
  } catch (err) {
    message.error(String(err));
  }
}
function ctxKill() {
  const row = ctxMenu.value?.row;
  closeCtx();
  if (!row) return;
  dialog.warning({
    title: t("btn.kill"),
    content: isCriticalProcess(row.pid, row.name)
      ? t("confirm.critical", { name: row.name, pid: row.pid })
      : t("confirm.kill", { name: row.name, pid: row.pid }),
    positiveText: t("btn.kill"),
    negativeText: t("btn.cancel"),
    onPositiveClick: () => handleKill(row.pid, row.creationTime100ns),
  });
}
async function handleKill(pid: number, creationTime100ns: string | null) {
  try {
    await store.kill(pid, creationTime100ns);
    message.success(t("toast.killed", { pid }));
  } catch (e) {
    // 受限模式下权限不足：当场引导提权；核心系统进程不误导用户去提权（ADR-0010）
    if (errorCode(e) === "ACCESS_DENIED" && !store.isElevated) {
      dialog.warning({
        title: t("elevate.title"),
        content: t("elevate.needAdmin"),
        positiveText: t("elevate.action"),
        negativeText: t("btn.cancel"),
        onPositiveClick: () => elevate(),
      });
    } else {
      message.error(killErrorMessage(t, e));
    }
  } finally {
    await store.refreshSnapshot();
  }
}

/* ---------- 指标展示：读不到时不伪装成 0 ---------- */
function metricText(value: number | null, status: MetricStatus, format: (v: number) => string): string {
  if (status === "sampling") return t("metric.sampling");
  if (status === "accessDenied") return t("metric.accessDenied");
  if (status === "processExited") return t("metric.processExited");
  if (status === "unavailable" || value === null) return "—";
  return format(value);
}
function formatMem(v: number): string {
  const mb = v / 1024 / 1024;
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(mb < 10 && mb > 0 ? 1 : 0)} MB`;
}
function memText(row: ProcessInfo): string {
  return metricText(row.memoryBytes, row.memoryStatus, formatMem);
}
function formatCpu(v: number): string {
  return `${v.toFixed(1)}%`;
}
function cpuText(row: ProcessInfo): string {
  return metricText(row.cpuPercent, row.cpuStatus, formatCpu);
}
function cpuColor(row: ProcessInfo): string {
  if (row.cpuPercent === null) return "var(--pv-text-faint)";
  if (row.cpuPercent >= 25) return "#DC2626";
  if (row.cpuPercent >= 8) return "#D97706";
  return "var(--pv-text-soft)";
}
function cpuBarWidth(row: ProcessInfo): string {
  const v = row.cpuPercent ?? 0;
  return `${Math.max(0, Math.min(100, v * 4))}%`;
}
function cpuBarColor(row: ProcessInfo): string {
  if (row.cpuPercent === null) return "var(--pv-text-faint)";
  if (row.cpuPercent >= 25) return "#DC2626";
  if (row.cpuPercent >= 8) return "#D97706";
  return "var(--pv-green)";
}
function iconStyle(row: ProcessInfo) {
  const dark = isDark.value;
  if (row.tcp > 0) return dark ? { bg: "#14532D", text: "#86EFAC" } : { bg: "#DCFCE7", text: "#166534" };
  if (row.udp > 0) return dark ? { bg: "#1E3A5F", text: "#93C5FD" } : { bg: "#DBEAFE", text: "#1D4ED8" };
  return dark ? { bg: "#2E2A45", text: "#C4B5FD" } : { bg: "#EDE9FE", text: "#5B21B6" };
}
</script>

<template>
  <div class="pv-card flex h-full flex-col overflow-hidden">
    <!-- 表头（随列表横向滚动同步） -->
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
          v-for="c in columns"
          :key="c.key"
          class="relative shrink-0 select-none"
          :style="{ width: widthOf(c.key) }"
        >
          <button
            v-if="c.sort"
            class="sort-head"
            :style="{ color: sortKey === c.sort ? 'var(--pv-green-text)' : 'inherit' }"
            @click="toggleSort(c.sort!)"
          >
            {{ c.label }}
            <span class="sort-arrow">
              {{ sortKey === c.sort ? (sortDir === "desc" ? "▼" : "▲") : "↕" }}
            </span>
          </button>
          <template v-else>{{ c.label }}</template>
          <span class="col-resizer" @mousedown.prevent.stop="startResize($event, c.key)"></span>
        </div>
      </div>
    </div>

    <!-- 列表滚动区 -->
    <div ref="listScrollEl" class="min-h-0 flex-1 overflow-auto" @scroll="onListScroll">
      <div v-if="rows.length" :style="{ width: innerWidth + 'px', minWidth: '100%' }">
        <template v-for="row in rows" :key="row.pid">
          <div
            class="pv-row flex min-h-[52px] items-center gap-3 px-5 py-2.5 transition-colors"
            :class="{ 'pv-row-expanded': expandedPid === row.pid }"
            :style="{ borderBottom: '1px solid var(--pv-border)' }"
            @contextmenu.prevent="openCtx($event, row)"
            @dblclick="onRowDblClick($event, row.pid)"
          >
            <!-- 进程：图标 + 名称（单击复制）+ 软件名 / PID（单击复制） -->
            <div class="flex min-w-0 items-center gap-2.5" :style="{ width: widthOf('process') }">
              <div
                class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg font-heading text-sm font-bold"
                :style="{
                  background: iconStyle(row).bg,
                  color: iconStyle(row).text,
                  border: '2px solid var(--pv-stroke)',
                }"
              >
                {{ row.name.charAt(0).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <button
                  class="pv-text-btn block w-full truncate text-left text-sm font-bold"
                  :style="{ color: 'var(--pv-text)' }"
                  :title="t('port.copyProcess')"
                  @click.stop="copy(row.name, t('port.col.process'))"
                >
                  {{ row.name }}
                </button>
                <div class="flex min-w-0 items-center gap-1 text-[11px]" :style="{ color: 'var(--pv-text-faint)' }">
                  <span class="min-w-0 truncate" :title="row.path">{{ row.software || row.path || "—" }}</span>
                  <span>·</span>
                  <button
                    class="pv-text-btn shrink-0 tabular-nums"
                    :title="t('port.copyPid')"
                    @click.stop="copy(String(row.pid), 'PID')"
                  >
                    {{ row.pid }}
                  </button>
                </div>
              </div>
            </div>

            <!-- CPU：数值 + 贴纸细条 -->
            <div class="cpu-cell" :style="{ width: widthOf('cpu') }" :title="`CPU ${cpuText(row)}`">
              <span class="cpu-value" :style="{ color: cpuColor(row) }">{{ cpuText(row) }}</span>
              <span class="cpu-track">
                <i :style="{ width: cpuBarWidth(row), background: cpuBarColor(row) }"></i>
              </span>
            </div>

            <!-- 内存工作集 -->
            <div
              class="tabular-nums text-sm"
              :style="{ width: widthOf('memory'), color: 'var(--pv-text-soft)' }"
              :title="row.memoryBytes === null ? memText(row) : `${row.memoryBytes.toLocaleString()} 字节`"
            >
              {{ memText(row) }}
            </div>

            <!-- 连接：TCP / UDP / 监听合并，去掉三个重复数字列与 pill -->
            <div class="flex items-center gap-2 text-xs font-bold" :style="{ width: widthOf('connections'), color: 'var(--pv-text)' }">
              <template v-if="row.tcp + row.udp > 0">
                <span class="flex items-center gap-1">
                  <i class="conn-dot" :style="{ background: 'var(--pv-green)' }"></i>
                  TCP {{ row.tcp }}
                </span>
                <span v-if="row.listening" class="flex items-center gap-1" :style="{ color: 'var(--pv-amber-text)' }">
                  <i class="conn-dot" :style="{ background: 'var(--pv-amber-text)' }"></i>
                  监听 {{ row.listening }}
                </span>
              </template>
              <span v-else :style="{ color: 'var(--pv-text-faint)' }">—</span>
            </div>

            <!-- 端口：扁平贴纸 chip，最多 3 个 + 余量 -->
            <div
              class="flex min-w-0 flex-nowrap items-center gap-1 overflow-hidden"
              :style="{ width: widthOf('ports') }"
            >
              <button
                v-for="p in row._ports.slice(0, 3)"
                :key="p"
                class="flat-port"
                :style="{ background: 'var(--pv-blue-bg)', color: 'var(--pv-blue-text)' }"
                :title="t('port.copyPort')"
                @click.stop="copy(String(p), t('port.col.local'))"
              >
                {{ p }}
              </button>
              <span
                v-if="row._ports.length > 3"
                class="shrink-0 text-[10px] font-bold tabular-nums"
                :style="{ color: 'var(--pv-text-faint)' }"
              >
                +{{ row._ports.length - 3 }}
              </span>
              <span v-if="!row._ports.length" class="text-[11px]" :style="{ color: 'var(--pv-text-faint)' }">—</span>
            </div>

            <!-- 操作（row-actions 标记用于双击行时排除，避免误触「连接 / 结束」） -->
            <div class="row-actions flex shrink-0 items-center gap-2" :style="{ width: widthOf('actions') }">
              <n-popconfirm
                :positive-text="t('btn.kill')"
                :negative-text="t('btn.cancel')"
                @positive-click="handleKill(row.pid, row.creationTime100ns)"
              >
                <template #trigger>
                  <button class="pv-mini-btn danger row-action-btn kill-action-btn">{{ t("btn.kill") }}</button>
                </template>
                <template #default>
                  {{
                    isCriticalProcess(row.pid, row.name)
                      ? t("confirm.critical", { name: row.name, pid: row.pid })
                      : t("confirm.kill", { name: row.name, pid: row.pid })
                  }}
                </template>
              </n-popconfirm>
              <button
                v-if="row.tcp + row.udp > 0"
                class="pv-mini-btn row-action-btn"
                :style="{ color: 'var(--pv-green-text)' }"
                @click="toggle(row.pid)"
              >
                {{ expandedPid === row.pid ? t("btn.collapse") : t("btn.connections") }}
              </button>
            </div>
          </div>

          <!-- 展开连接明细 -->
          <div
            v-if="expandedPid === row.pid && row.tcp + row.udp > 0"
            class="px-5 py-3"
            :style="{ background: 'var(--pv-card-2)', borderBottom: '1px solid var(--pv-border)' }"
          >
            <div class="flex flex-col gap-1">
              <div
                v-for="(c, i) in store.connectionsOf(row.pid)"
                :key="i"
                class="flex items-center gap-4 rounded-lg px-3 py-1.5 text-xs"
                :style="{ background: 'var(--pv-card)', border: '1.5px solid var(--pv-border)' }"
              >
                <span class="w-14 shrink-0 font-extrabold" :style="{ color: 'var(--pv-text)' }">{{ c.protocol }}</span>
                <span class="w-20 shrink-0 tabular-nums" :style="{ color: 'var(--pv-text-soft)' }">:{{ c.localPort }}</span>
                <span class="min-w-0 flex-1 truncate tabular-nums" :style="{ color: 'var(--pv-text-soft)' }">{{ c.localAddress }}</span>
                <span class="min-w-0 flex-1 truncate tabular-nums" :style="{ color: 'var(--pv-text-soft)' }">{{ c.remoteAddress }}</span>
                <span
                  class="shrink-0 rounded-full px-2 py-px text-[10px] font-bold"
                  :style="{
                    background: c.state === 'LISTENING' ? 'var(--pv-green-bg)' : 'var(--pv-border)',
                    color: c.state === 'LISTENING' ? 'var(--pv-green-text)' : 'var(--pv-text-soft)',
                    border: '1.5px solid var(--pv-stroke)',
                  }"
                >
                  {{ c.state }}
                </span>
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- 空状态 -->
      <div v-else class="flex h-full flex-col items-center justify-center gap-3">
        <div
          class="flex h-16 w-16 items-center justify-center rounded-2xl font-heading text-3xl font-bold"
          :style="{
            background: 'var(--pv-blue-bg)',
            color: 'var(--pv-blue-text)',
            border: '3px solid var(--pv-stroke)',
            boxShadow: '3px 3px 0 var(--pv-stroke)',
          }"
        >
          ∅
        </div>
        <div class="text-sm font-bold" :style="{ color: 'var(--pv-text-soft)' }">{{ t("empty.title") }}</div>
        <div class="text-xs" :style="{ color: 'var(--pv-text-faint)' }">{{ t("empty.tip") }}</div>
      </div>
    </div>

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div
        v-if="ctxMenu"
        class="pv-ctx"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @contextmenu.prevent
        @click.stop
      >
        <button class="pv-ctx-item" @click="ctxOpenLocation">
          <svg class="pv-ctx-ico" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z" />
            <path d="M3 10h18" />
          </svg>
          {{ t("ctx.openLocation") }}
        </button>
        <button class="pv-ctx-item pv-ctx-danger" @click="ctxKill">
          <svg class="pv-ctx-ico" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true">
            <path d="M6 6l12 12M18 6L6 18" />
          </svg>
          {{ t("btn.kill") }}
        </button>
      </div>
    </Teleport>

    <!-- 进程详情抽屉 -->
    <ProcessDetail v-model:show="showDetail" :pid="detailPid" />
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
  border: none;
  background: none;
  cursor: pointer;
  font: inherit;
  color: inherit;
  letter-spacing: inherit;
}
.sort-arrow {
  font-size: 9px;
  line-height: 1;
  opacity: 0.45;
  transition: opacity 0.12s ease-out;
}
.sort-head:hover .sort-arrow {
  opacity: 1;
}

/* CPU 细条：数值在上、进度在下，不抢列宽 */
.cpu-cell {
  display: flex;
  flex-direction: column;
  gap: 5px;
  flex: none;
}
.cpu-value {
  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  line-height: 1.1;
}
.cpu-track {
  display: block;
  height: 3px;
  border-radius: 999px;
  background: var(--pv-border);
  overflow: hidden;
}
.cpu-track i {
  display: block;
  height: 100%;
  border-radius: 999px;
}
.conn-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex: none;
}
.flat-port {
  border: 0;
  border-radius: 999px;
  padding: 2px 7px;
  font-size: 10px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  cursor: pointer;
  flex: none;
}
.flat-port:hover {
  filter: brightness(0.96);
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

.pv-ctx {
  position: fixed;
  z-index: 3000;
  min-width: 180px;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  background: var(--pv-card);
  border: 3px solid var(--pv-stroke);
  border-radius: 12px;
  box-shadow: var(--pv-sticker-shadow-sm);
}
.pv-ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: none;
  border-radius: 8px;
  background: transparent;
  font: inherit;
  font-size: 13px;
  font-weight: 700;
  color: var(--pv-text);
  cursor: pointer;
  text-align: left;
}
.pv-ctx-item:hover {
  background: var(--pv-green-bg);
}
.pv-ctx-ico {
  width: 16px;
  height: 16px;
  flex: none;
}
.pv-ctx-danger {
  color: #dc2626;
}
.pv-ctx-danger:hover {
  background: #fee2e2;
}

.pv-row {
  background: transparent;
}
.pv-row:hover {
  background: var(--pv-row-hover);
  box-shadow: inset 3px 0 0 var(--pv-green);
}
.pv-row-expanded {
  background: var(--pv-card-2);
}
.pv-row-expanded:hover {
  background: var(--pv-row-hover);
}
</style>
