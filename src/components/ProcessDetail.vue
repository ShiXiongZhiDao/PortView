<script setup lang="ts">
import { computed } from "vue";
import { useMessage, useDialog } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useConnectionsStore } from "../stores/connections";
import { useClipboard } from "../composables/useClipboard";
import { useElevation } from "../composables/useElevation";
import { useI18n } from "../i18n";
import { errorCode, killErrorMessage } from "../lib/kill-error";
import { isCriticalProcess } from "../lib/process-safety";
import type { MetricStatus } from "../types";

const props = defineProps<{ show: boolean; pid: number | null }>();
const emit = defineEmits<{ (e: "update:show", v: boolean): void }>();

const store = useConnectionsStore();
const { t } = useI18n();
const message = useMessage();
const dialog = useDialog();
const { copy } = useClipboard();
const { elevate } = useElevation();

const proc = computed(() => store.processes.find((p) => p.pid === props.pid) ?? null);
const conns = computed(() => (props.pid === null ? [] : store.connectionsOf(props.pid)));

function metric(v: number | null, status: MetricStatus, fmt: (n: number) => string): string {
  if (status === "sampling") return t("metric.sampling");
  if (status === "accessDenied") return t("metric.accessDenied");
  if (status === "processExited") return t("metric.processExited");
  if (status === "unavailable" || v === null) return "—";
  return fmt(v);
}
const memoryText = computed(() =>
  proc.value ? metric(proc.value.memoryBytes, proc.value.memoryStatus, (n) => `${(n / 1024 / 1024).toFixed(1)} MB`) : "—",
);
const cpuText = computed(() =>
  proc.value ? metric(proc.value.cpuPercent, proc.value.cpuStatus, (n) => `${n.toFixed(1)}%`) : "—",
);
const createdText = computed(() => {
  const raw = proc.value?.creationTime100ns;
  if (!raw) return "—";
  // FILETIME（100ns，1601-01-01 起）→ Unix 毫秒
  const ms = Number(raw) / 10000 - 11644473600000;
  const d = new Date(ms);
  return Number.isNaN(d.getTime()) ? "—" : d.toLocaleString();
});

async function openLocation() {
  if (!proc.value?.path) {
    message.warning(t("ctx.noPath"));
    return;
  }
  try {
    await invoke("open_in_explorer", { path: proc.value.path });
  } catch (err) {
    message.error(String(err));
  }
}

function kill() {
  const p = proc.value;
  if (!p) return;
  dialog.warning({
    title: t("btn.kill"),
    content: isCriticalProcess(p.pid, p.name)
      ? t("confirm.critical", { name: p.name, pid: p.pid })
      : t("confirm.kill", { name: p.name, pid: p.pid }),
    positiveText: t("btn.kill"),
    negativeText: t("btn.cancel"),
    onPositiveClick: async () => {
      try {
        await store.kill(p.pid, p.creationTime100ns);
        message.success(t("toast.killed", { pid: p.pid }));
        emit("update:show", false);
      } catch (e) {
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
    },
  });
}
</script>

<template>
  <n-drawer :show="show" :width="520" :on-update:show="(v: boolean) => emit('update:show', v)" placement="right">
    <n-drawer-content :title="t('detail.title')" closable>
      <div v-if="proc" class="flex flex-col gap-4">
        <!-- 标识 -->
        <div class="flex items-center gap-3">
          <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl font-heading text-xl font-bold"
            :style="{ background: 'var(--pv-green-bg)', color: 'var(--pv-green-text)', border: '2px solid var(--pv-stroke)' }">
            {{ proc.name.charAt(0).toUpperCase() }}
          </div>
          <div class="min-w-0">
            <div class="truncate text-base font-bold" :style="{ color: 'var(--pv-text)' }">{{ proc.name }}</div>
            <div class="truncate text-xs" :style="{ color: 'var(--pv-text-soft)' }">{{ proc.software || '—' }}</div>
          </div>
        </div>

        <!-- 关键指标 -->
        <div class="grid grid-cols-2 gap-2">
          <div class="rounded-lg px-3 py-2" :style="{ background: 'var(--pv-card-2)', border: '2px solid var(--pv-stroke)' }">
            <div class="text-[10px] font-bold" :style="{ color: 'var(--pv-text-faint)' }">PID</div>
            <button class="text-sm font-bold tabular-nums hover:underline" :style="{ color: 'var(--pv-text)' }" @click="copy(String(proc.pid), 'PID')">{{ proc.pid }}</button>
          </div>
          <div class="rounded-lg px-3 py-2" :style="{ background: 'var(--pv-card-2)', border: '2px solid var(--pv-stroke)' }">
            <div class="text-[10px] font-bold" :style="{ color: 'var(--pv-text-faint)' }">CPU</div>
            <div class="text-sm font-bold tabular-nums" :style="{ color: 'var(--pv-text)' }">{{ cpuText }}</div>
          </div>
          <div class="rounded-lg px-3 py-2" :style="{ background: 'var(--pv-card-2)', border: '2px solid var(--pv-stroke)' }">
            <div class="text-[10px] font-bold" :style="{ color: 'var(--pv-text-faint)' }">{{ t('list.col.memory') }}</div>
            <div class="text-sm font-bold tabular-nums" :style="{ color: 'var(--pv-text)' }">{{ memoryText }}</div>
          </div>
          <div class="rounded-lg px-3 py-2" :style="{ background: 'var(--pv-card-2)', border: '2px solid var(--pv-stroke)' }">
            <div class="text-[10px] font-bold" :style="{ color: 'var(--pv-text-faint)' }">{{ t('detail.created') }}</div>
            <div class="truncate text-xs tabular-nums" :style="{ color: 'var(--pv-text)' }" :title="createdText">{{ createdText }}</div>
          </div>
        </div>

        <!-- 连接统计 -->
        <div class="flex gap-2 text-xs font-bold">
          <span class="rounded-full px-3 py-1" :style="{ background: 'var(--pv-green-bg)', color: 'var(--pv-green-text)' }">TCP {{ proc.tcp }}</span>
          <span class="rounded-full px-3 py-1" :style="{ background: 'var(--pv-blue-bg)', color: 'var(--pv-blue-text)' }">UDP {{ proc.udp }}</span>
          <span class="rounded-full px-3 py-1" :style="{ background: 'var(--pv-amber-bg)', color: 'var(--pv-amber-text)' }">{{ t('stat.listening') }} {{ proc.listening }}</span>
        </div>

        <!-- 路径 -->
        <div v-if="proc.path" class="flex flex-col gap-1">
          <span class="text-[11px] font-extrabold" :style="{ color: 'var(--pv-text-soft)' }">{{ t('detail.path') }}</span>
          <div class="flex items-center gap-2">
            <span class="min-w-0 flex-1 truncate text-xs tabular-nums" :style="{ color: 'var(--pv-text-soft)' }" :title="proc.path">{{ proc.path }}</span>
            <button class="pv-btn px-2 py-1 text-[10px]" :style="{ background: 'var(--pv-card-2)', color: 'var(--pv-text)' }" @click="copy(proc.path, t('detail.path'))">{{ t('detail.copy') }}</button>
            <button class="pv-btn px-2 py-1 text-[10px]" :style="{ background: 'var(--pv-card-2)', color: 'var(--pv-text)' }" @click="openLocation">{{ t('ctx.openLocation') }}</button>
          </div>
        </div>

        <!-- 连接明细 -->
        <div class="flex flex-col gap-1">
          <span class="text-[11px] font-extrabold" :style="{ color: 'var(--pv-text-soft)' }">{{ t('detail.connections') }} ({{ conns.length }})</span>
          <div v-for="(c, i) in conns" :key="i" class="flex items-center gap-2 rounded-lg px-2 py-1 text-[11px] tabular-nums"
            :style="{ background: 'var(--pv-card-2)', border: '1.5px solid var(--pv-border)' }">
            <span class="w-10 font-bold" :style="{ color: 'var(--pv-text)' }">{{ c.protocol }}</span>
            <button class="min-w-0 flex-1 truncate text-left hover:underline" :style="{ color: 'var(--pv-text-soft)' }" @click="copy(c.localAddress, t('port.col.localAddr'))">{{ c.localAddress }}</button>
            <span class="min-w-0 flex-1 truncate" :style="{ color: 'var(--pv-text-faint)' }">{{ c.remoteAddress }}</span>
            <span class="shrink-0" :style="{ color: 'var(--pv-text-soft)' }">{{ c.state }}</span>
          </div>
          <div v-if="!conns.length" class="text-xs" :style="{ color: 'var(--pv-text-faint)' }">—</div>
        </div>

        <!-- 结束进程 -->
        <button class="pv-btn mt-2 flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-bold text-white"
          :style="{ background: '#DC2626' }" @click="kill">
          {{ t('btn.kill') }}
        </button>
      </div>
      <div v-else class="text-sm" :style="{ color: 'var(--pv-text-soft)' }">{{ t('detail.notFound') }}</div>
    </n-drawer-content>
  </n-drawer>
</template>
