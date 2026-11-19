<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDialog, useMessage } from "naive-ui";
import { useConnectionsStore } from "../stores/connections";
import { useTheme } from "../composables/useTheme";
import { useI18n } from "../i18n";
import type { ProcessInfo } from "../types";

const store = useConnectionsStore();
const message = useMessage();
const dialog = useDialog();
const { isDark } = useTheme();
const { t } = useI18n();

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

const rows = computed<(ProcessInfo & { _ports: number[] })[]>(() => {
  const list = [...(store.filteredProcesses as (ProcessInfo & { _ports: number[] })[])];
  const key = sortKey.value;
  const dir = sortDir.value === "desc" ? -1 : 1;
  return list.sort((a, b) => dir * (a[key] - b[key]));
});

/* ---------- 列定义与可拖拽列宽（所有列均可调整） ---------- */
type ColKey =
  | "process"
  | "pid"
  | "cpu"
  | "memory"
  | "tcp"
  | "udp"
  | "listening"
  | "ports"
  | "status"
  | "actions";

const COL_DEFAULTS: Record<ColKey, { width: number; min: number }> = {
  process: { width: 220, min: 130 },
  pid: { width: 70, min: 48 },
  cpu: { width: 76, min: 52 },
  memory: { width: 96, min: 60 },
  tcp: { width: 56, min: 40 },
  udp: { width: 56, min: 40 },
  listening: { width: 56, min: 40 },
  ports: { width: 220, min: 110 },
  status: { width: 86, min: 60 },
  actions: { width: 150, min: 132 },
};

/** 读取持久化列宽（损坏/越界数据自动回退） */
function loadColWidths(): Partial<Record<ColKey, number>> {
  try {
    const raw = JSON.parse(localStorage.getItem("port-view-col-widths") || "{}") as Record<string, unknown>;
    const out: Partial<Record<ColKey, number>> = {};
    (Object.keys(COL_DEFAULTS) as ColKey[]).forEach((k) => {
      const v = Number(raw[k]);
      if (Number.isFinite(v) && v > 0) {
        out[k] = Math.min(Math.max(v, COL_DEFAULTS[k].min), 600);
      }
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

/** 全部列像素宽度之和 */
const totalWidth = computed(() =>
  (Object.keys(COL_DEFAULTS) as ColKey[]).reduce((sum, k) => sum + colWidths[k], 0),
);

const columns = computed<{ key: ColKey; label: string; sort?: SortKey }[]>(() => [
  { key: "process", label: t("list.col.process") },
  { key: "pid", label: t("list.col.pid") },
  { key: "cpu", label: t("list.col.cpu"), sort: "cpu" },
  { key: "memory", label: t("list.col.memory"), sort: "memory" },
  { key: "tcp", label: t("list.col.tcp") },
  { key: "udp", label: t("list.col.udp") },
  { key: "listening", label: t("list.col.listening") },
  { key: "ports", label: t("list.col.ports") },
  { key: "status", label: t("list.col.status") },
  { key: "actions", label: t("list.col.actions") },
]);

/** 表头/行共用布局参数（与 class gap-3、px-5 保持一致） */
const CELL_GAP = 12;
const H_PADDING = 40;
const innerWidth = computed(() => totalWidth.value + (columns.value.length - 1) * CELL_GAP + H_PADDING);

/** 拖拽调整列宽 */
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
    localStorage.setItem("port-view-col-widths", JSON.stringify(colWidths));
  } catch {
    /* 持久化失败忽略 */
  }
}

/* ---------- 表头与列表横向滚动同步 ---------- */
const headerScrollEl = ref<HTMLElement | null>(null);
const listScrollEl = ref<HTMLElement | null>(null);
function onListScroll() {
  const el = listScrollEl.value;
  if (headerScrollEl.value && el) {
    headerScrollEl.value.scrollLeft = el.scrollLeft;
  }
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
    content: isCritical(row.pid, row.name)
      ? t("confirm.critical", { name: row.name, pid: row.pid })
      : t("confirm.kill", { name: row.name, pid: row.pid }),
    positiveText: t("btn.kill"),
    negativeText: t("btn.cancel"),
    onPositiveClick: () => handleKill(row.pid),
  });
}

async function handleKill(pid: number) {
  try {
    const result = await store.kill(pid);
    message.success(result);
  } catch (e) {
    message.error(String(e));
  } finally {
    await store.refresh();
  }
}

function isCritical(pid: number, name: string): boolean {
  const n = name.toLowerCase();
  return (
    pid <= 4 ||
    n === "system" ||
    n === "svchost.exe" ||
    n === "csrss.exe" ||
    n === "wininit.exe" ||
    n === "services.exe" ||
    n === "lsass.exe"
  );
}

/** 字节数格式化为可读内存（MB / GB） */
function formatMem(bytes: number): string {
  if (!bytes) return "0 MB";
  const mb = bytes / 1024 / 1024;
  if (mb >= 1024) return `${(mb / 1024).toFixed(2)} GB`;
  return `${mb.toFixed(mb < 10 ? 1 : 0)} MB`;
}

/** CPU 占用百分比文本 */
function formatCpu(cpu: number): string {
  return `${(cpu ?? 0).toFixed(1)}%`;
}

/** CPU 占用越高颜色越暖，便于快速定位吃 CPU 的进程 */
function cpuColor(cpu: number): string {
  if (cpu >= 25) return "#DC2626";
  if (cpu >= 8) return "#D97706";
  return "var(--pv-text-soft)";
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

    <!-- 列表滚动区（垂直 + 水平） -->
    <div ref="listScrollEl" class="min-h-0 flex-1 overflow-auto" @scroll="onListScroll">
      <div v-if="rows.length" :style="{ width: innerWidth + 'px', minWidth: '100%' }">
        <template v-for="row in rows" :key="row.pid">
          <!-- 单行 -->
          <div
            class="flex items-center gap-3 px-5 py-2.5 transition-colors"
            :style="{
              borderBottom: '1px dashed var(--pv-border)',
              background: expandedPid === row.pid ? 'var(--pv-card-2)' : 'transparent',
            }"
            @contextmenu.prevent="openCtx($event, row)"
          >
            <!-- 图标 + 名称 -->
            <div class="flex min-w-0 items-center gap-2.5" :style="{ width: widthOf('process') }">
              <div
                class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg font-heading text-sm font-bold"
                :style="{
                  background: iconStyle(row).bg,
                  color: iconStyle(row).text,
                  border: '2px solid var(--pv-stroke)',
                }"
              >
                {{ row.name.charAt(0).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <div class="truncate text-sm font-bold" :style="{ color: 'var(--pv-text)' }" :title="row.name">
                  {{ row.name }}
                </div>
                <div class="truncate text-[11px]" :style="{ color: 'var(--pv-text-faint)' }" :title="row.path">
                  {{ row.software || row.path || "—" }}
                </div>
              </div>
            </div>

            <div class="tabular-nums text-sm" :style="{ width: widthOf('pid'), color: 'var(--pv-text-soft)' }">
              {{ row.pid }}
            </div>

            <!-- CPU 占用 -->
            <div
              class="tabular-nums text-sm font-bold"
              :style="{ width: widthOf('cpu'), color: cpuColor(row.cpu) }"
              :title="`CPU ${formatCpu(row.cpu)}`"
            >
              {{ formatCpu(row.cpu) }}
            </div>

            <!-- 内存工作集 -->
            <div
              class="tabular-nums text-sm"
              :style="{ width: widthOf('memory'), color: 'var(--pv-text-soft)' }"
              :title="`${row.memory.toLocaleString()} 字节`"
            >
              {{ formatMem(row.memory) }}
            </div>

            <div class="tabular-nums text-sm font-bold" :style="{ width: widthOf('tcp'), color: 'var(--pv-text)' }">
              {{ row.tcp }}
            </div>
            <div class="tabular-nums text-sm font-bold" :style="{ width: widthOf('udp'), color: 'var(--pv-text)' }">
              {{ row.udp }}
            </div>
            <div
              class="tabular-nums text-sm font-bold"
              :style="{ width: widthOf('listening'), color: 'var(--pv-text)' }"
            >
              {{ row.listening }}
            </div>

            <!-- 端口 -->
            <div
              class="flex min-w-0 flex-wrap items-center gap-1 overflow-hidden"
              :style="{ width: widthOf('ports') }"
            >
              <span
                v-for="p in row._ports.slice(0, 6)"
                :key="p"
                class="rounded-full px-2 py-px text-[10px] font-bold tabular-nums"
                :style="{
                  background: 'var(--pv-blue-bg)',
                  color: 'var(--pv-blue-text)',
                  border: '1.5px solid var(--pv-stroke)',
                }"
              >
                {{ p }}
              </span>
              <span
                v-if="row._ports.length > 6"
                class="rounded-full px-2 py-px text-[10px] font-bold"
                :style="{ background: 'var(--pv-border)', color: 'var(--pv-text-soft)', border: '1.5px solid var(--pv-stroke)' }"
              >
                +{{ row._ports.length - 6 }}
              </span>
              <span v-if="!row._ports.length" class="text-[11px]" :style="{ color: 'var(--pv-text-faint)' }">—</span>
            </div>

            <!-- 状态 -->
            <div :style="{ width: widthOf('status') }">
              <span
                class="pv-pill text-[10px]"
                :style="{
                  background: row.tcp + row.udp > 0 ? 'var(--pv-green-bg)' : 'var(--pv-border)',
                  color: row.tcp + row.udp > 0 ? 'var(--pv-green-text)' : 'var(--pv-text-soft)',
                  padding: '1px 8px',
                }"
              >
                {{ row.tcp + row.udp > 0 ? t("status.active") : t("status.idle") }}
              </span>
            </div>

            <!-- 操作 -->
            <div class="flex shrink-0 items-center gap-1.5" :style="{ width: widthOf('actions') }">
              <button
                v-if="row.tcp + row.udp > 0"
                class="pv-btn px-2.5 py-1 text-[11px]"
                :style="{ background: 'var(--pv-green-bg)', color: 'var(--pv-green-text)' }"
                @click="toggle(row.pid)"
              >
                {{ expandedPid === row.pid ? t("btn.collapse") : t("btn.connections") }}
              </button>
              <n-popconfirm
                :positive-text="t('btn.kill')"
                :negative-text="t('btn.cancel')"
                @positive-click="handleKill(row.pid)"
              >
                <template #trigger>
                  <button class="pv-btn px-2.5 py-1 text-[11px] text-white" :style="{ background: '#DC2626' }">
                    {{ t("btn.kill") }}
                  </button>
                </template>
                <template #default>
                  {{
                    isCritical(row.pid, row.name)
                      ? t("confirm.critical", { name: row.name, pid: row.pid })
                      : t("confirm.kill", { name: row.name, pid: row.pid })
                  }}
                </template>
              </n-popconfirm>
            </div>
          </div>

          <!-- 展开连接明细 -->
          <div
            v-if="expandedPid === row.pid && row.tcp + row.udp > 0"
            class="px-5 py-3"
            :style="{ background: 'var(--pv-card-2)', borderBottom: '1px dashed var(--pv-border)' }"
          >
            <div class="flex flex-col gap-1">
              <div
                v-for="(c, i) in store.connectionsOf(row.pid)"
                :key="i"
                class="flex items-center gap-4 rounded-lg px-3 py-1.5 text-xs"
                :style="{ background: 'var(--pv-card)', border: '1.5px solid var(--pv-border)' }"
              >
                <span class="w-14 shrink-0 font-extrabold" :style="{ color: 'var(--pv-text)' }">{{ c.protocol }}</span>
                <span class="w-20 shrink-0 tabular-nums" :style="{ color: 'var(--pv-text-soft)' }">
                  :{{ c.localPort }}
                </span>
                <span class="min-w-0 flex-1 truncate tabular-nums" :style="{ color: 'var(--pv-text-soft)' }">
                  {{ c.localAddress }}
                </span>
                <span class="min-w-0 flex-1 truncate tabular-nums" :style="{ color: 'var(--pv-text-soft)' }">
                  {{ c.remoteAddress }}
                </span>
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
  </div>
</template>

<style scoped>
/* 表头容器隐藏横向滚动条（WebView2 为 Chromium） */
.header-scroll {
  scrollbar-width: none;
}
.header-scroll::-webkit-scrollbar {
  display: none;
}

/* 可排序表头（CPU / 内存） */
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

/* 列宽拖拽手柄（吸附在列右边框，向两侧延伸 4px） */
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

/* 右键菜单（贴纸风格，Teleport 到 body 后仍继承 CSS 变量） */
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
</style>
