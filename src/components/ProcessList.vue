<script setup lang="ts">
import { computed, ref } from "vue";
import { NPopconfirm, useMessage } from "naive-ui";
import { useConnectionsStore } from "../stores/connections";
import { useTheme } from "../composables/useTheme";
import { useI18n } from "../i18n";
import type { ProcessInfo } from "../types";

const store = useConnectionsStore();
const message = useMessage();
const { isDark } = useTheme();
const { t } = useI18n();

/** 当前展开的进程 PID（同时只展开一个） */
const expandedPid = ref<number | null>(null);
function toggle(pid: number) {
  expandedPid.value = expandedPid.value === pid ? null : pid;
}

/** 可排序字段与方向：默认按内存从大到小 */
type SortKey = "memory" | "cpu";
const sortKey = ref<SortKey>("memory");
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

function iconStyle(row: ProcessInfo) {
  const dark = isDark.value;
  if (row.tcp > 0) return dark ? { bg: "#14532D", text: "#86EFAC" } : { bg: "#DCFCE7", text: "#166534" };
  if (row.udp > 0) return dark ? { bg: "#1E3A5F", text: "#93C5FD" } : { bg: "#DBEAFE", text: "#1D4ED8" };
  return dark ? { bg: "#2E2A45", text: "#C4B5FD" } : { bg: "#EDE9FE", text: "#5B21B6" };
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

const columns = computed<{ label: string; width: string; sort?: SortKey }[]>(() => [
  { label: t("list.col.process"), width: "180px" },
  { label: t("list.col.pid"), width: "64px" },
  { label: t("list.col.cpu"), width: "60px", sort: "cpu" },
  { label: t("list.col.memory"), width: "84px", sort: "memory" },
  { label: t("list.col.tcp"), width: "48px" },
  { label: t("list.col.udp"), width: "48px" },
  { label: t("list.col.listening"), width: "48px" },
  { label: t("list.col.ports"), width: "1fr" },
  { label: t("list.col.status"), width: "72px" },
  { label: t("list.col.actions"), width: "140px" },
]);
</script>

<template>
  <div class="pv-card flex h-full flex-col overflow-hidden">
    <!-- 表头 -->
    <div
      class="flex shrink-0 items-center gap-3 px-5 py-3 text-xs font-extrabold tracking-wider"
      :style="{ background: 'var(--pv-card-2)', borderBottom: '3px solid var(--pv-stroke)' }"
    >
      <div v-for="c in columns" :key="c.label" :style="{ width: c.width, flex: c.width === '1fr' ? 1 : 'none' }">
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
      </div>
    </div>

    <!-- 列表滚动区 -->
    <div v-if="rows.length" class="min-h-0 flex-1 overflow-y-auto">
      <template v-for="row in rows" :key="row.pid">
        <!-- 单行 -->
        <div
          class="flex items-center gap-3 px-5 py-2.5 transition-colors"
          :style="{
            borderBottom: '1px dashed var(--pv-border)',
            background: expandedPid === row.pid ? 'var(--pv-card-2)' : 'transparent',
          }"
        >
          <!-- 图标 + 名称 -->
          <div class="flex min-w-0 items-center gap-2.5" style="width: 180px">
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

          <div class="tabular-nums text-sm" style="width: 64px" :style="{ color: 'var(--pv-text-soft)' }">
            {{ row.pid }}
          </div>

          <!-- CPU 占用 -->
          <div
            class="tabular-nums text-sm font-bold"
            style="width: 60px"
            :style="{ color: cpuColor(row.cpu) }"
            :title="`CPU ${formatCpu(row.cpu)}`"
          >
            {{ formatCpu(row.cpu) }}
          </div>

          <!-- 内存工作集 -->
          <div
            class="tabular-nums text-sm"
            style="width: 84px"
            :style="{ color: 'var(--pv-text-soft)' }"
            :title="`${row.memory.toLocaleString()} 字节`"
          >
            {{ formatMem(row.memory) }}
          </div>

          <div class="tabular-nums text-sm font-bold" style="width: 48px" :style="{ color: 'var(--pv-text)' }">
            {{ row.tcp }}
          </div>
          <div class="tabular-nums text-sm font-bold" style="width: 48px" :style="{ color: 'var(--pv-text)' }">
            {{ row.udp }}
          </div>
          <div class="tabular-nums text-sm font-bold" style="width: 48px" :style="{ color: 'var(--pv-text)' }">
            {{ row.listening }}
          </div>

          <!-- 端口 -->
          <div class="flex min-w-0 flex-1 flex-wrap items-center gap-1 overflow-hidden">
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
          <div style="width: 72px">
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
          <div class="flex shrink-0 items-center gap-1.5" style="width: 140px">
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
    <div v-else class="flex flex-1 flex-col items-center justify-center gap-3">
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
</template>

<style scoped>
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
</style>
