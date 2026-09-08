<script setup lang="ts">
import { computed } from "vue";
import { useThemeVars } from "naive-ui";
import { useConnectionsStore } from "../stores/connections";

const store = useConnectionsStore();
const themeVars = useThemeVars();

const protocolOptions = [
  { label: "全部", value: "全部" },
  { label: "TCP", value: "TCP" },
  { label: "UDP", value: "UDP" },
];

const statusOptions = [
  { label: "全部", value: "全部" },
  { label: "LISTENING", value: "LISTENING" },
  { label: "ESTABLISHED", value: "ESTABLISHED" },
  { label: "TIME_WAIT", value: "TIME_WAIT" },
  { label: "CLOSE_WAIT", value: "CLOSE_WAIT" },
  { label: "其他", value: "其他" },
];

const updatedText = computed(() => {
  if (!store.lastUpdated) return "--:--:--";
  const d = new Date(store.lastUpdated);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
});
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 pt-4">
    <n-input
      v-model:value="store.search"
      clearable
      size="medium"
      placeholder="搜索：端口、进程名、PID、路径、软件名、拼音"
      class="w-[380px]"
    />
    <n-select v-model:value="store.protocol" :options="protocolOptions" class="w-24" />
    <n-select v-model:value="store.status" :options="statusOptions" class="w-40" />
    <div class="ml-auto whitespace-nowrap text-xs tabular-nums" :style="{ color: themeVars.textColor3 }">
      当前显示 {{ store.filteredRows.length }} 条，{{ updatedText }} 更新
    </div>
  </div>
</template>
