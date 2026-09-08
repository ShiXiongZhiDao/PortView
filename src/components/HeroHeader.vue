<script setup lang="ts">
import { computed } from "vue";
import { useConnectionsStore } from "../stores/connections";
import { useTheme } from "../composables/useTheme";

const store = useConnectionsStore();
const { themeMode, setTheme } = useTheme();

const themeOptions = [
  { label: "跟随系统", value: "system" },
  { label: "暗色", value: "dark" },
  { label: "亮色", value: "light" },
];

/** Hero 右侧统计块（彩色贴纸小块） */
const stats = computed(() => [
  { label: "TCP", value: store.stats.tcp, bg: "var(--pv-green-bg)", text: "var(--pv-green-text)" },
  { label: "UDP", value: store.stats.udp, bg: "var(--pv-blue-bg)", text: "var(--pv-blue-text)" },
  { label: "监听", value: store.stats.listening, bg: "var(--pv-amber-bg)", text: "var(--pv-amber-text)" },
  { label: "进程", value: store.stats.procs, bg: "var(--pv-violet-bg)", text: "var(--pv-violet-text)" },
]);
</script>

<template>
  <div class="pv-card flex shrink-0 items-center justify-between gap-6 px-8 py-5">
    <!-- 左侧：品牌 -->
    <div class="flex items-center gap-4">
      <div
        class="flex h-14 w-14 shrink-0 items-center justify-center font-heading text-xl font-bold text-white"
        :style="{
          background: 'var(--pv-green)',
          border: '3px solid var(--pv-stroke)',
          borderRadius: '14px',
          boxShadow: '3px 3px 0 var(--pv-stroke)',
        }"
      >
        PV
      </div>
      <div class="flex flex-col gap-1.5">
        <h1 class="font-heading text-[40px] font-bold leading-none" :style="{ color: 'var(--pv-text)' }">
          port-view
        </h1>
        <p class="text-sm" :style="{ color: 'var(--pv-text-soft)' }">
          Windows Port &amp; Process Monitor · 实时监控端口与进程
        </p>
      </div>
    </div>

    <!-- 右侧：统计块 + 主题切换 -->
    <div class="flex shrink-0 items-center gap-3">
      <div
        v-for="s in stats"
        :key="s.label"
        class="flex flex-col items-center gap-0.5 rounded-xl px-4 py-2"
        :style="{ background: s.bg, border: '2px solid var(--pv-stroke)', minWidth: '68px' }"
      >
        <span class="font-heading text-2xl font-bold leading-none tabular-nums" :style="{ color: s.text }">
          {{ s.value }}
        </span>
        <span class="text-[11px] font-extrabold tracking-wider" :style="{ color: s.text }">
          {{ s.label }}
        </span>
      </div>
      <n-select :value="themeMode" :options="themeOptions" class="ml-2 w-28" size="large" @update:value="setTheme" />
    </div>
  </div>
</template>
