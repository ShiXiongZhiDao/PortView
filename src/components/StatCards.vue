<script setup lang="ts">
import { computed } from "vue";
import { useThemeVars } from "naive-ui";
import { useConnectionsStore } from "../stores/connections";

const store = useConnectionsStore();
const themeVars = useThemeVars();

const cards = computed(() => [
  { label: "TCP", value: store.stats.tcp, color: "#2080f0" },
  { label: "UDP", value: store.stats.udp, color: "#8a2be2" },
  { label: "监听端口", value: store.stats.listening, color: "#f0a020" },
  { label: "进程", value: store.stats.procs, color: "#18a058" },
]);
</script>

<template>
  <div class="grid grid-cols-4 gap-4">
    <div
      v-for="c in cards"
      :key="c.label"
      class="flex items-center gap-4 rounded-xl border px-5 py-4"
      :style="{
        borderColor: themeVars.borderColor,
        background: themeVars.cardColor,
      }"
    >
      <div
        class="flex h-11 w-11 shrink-0 items-center justify-center rounded-lg text-base font-bold"
        :style="{ background: c.color + '22', color: c.color }"
      >
        {{ c.label.slice(0, 1) }}
      </div>
      <div class="min-w-0">
        <div class="text-xs" :style="{ color: themeVars.textColor3 }">{{ c.label }}</div>
        <div class="text-2xl font-semibold leading-tight tabular-nums" :style="{ color: c.color }">
          {{ c.value }}
        </div>
      </div>
    </div>
  </div>
</template>
