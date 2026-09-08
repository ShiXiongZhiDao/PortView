<script setup lang="ts">
import { useConnectionsStore } from "../stores/connections";

const store = useConnectionsStore();

const protocolOptions = [
  { label: "全部协议", value: "全部" },
  { label: "TCP", value: "TCP" },
  { label: "UDP", value: "UDP" },
];

const statusOptions = [
  { label: "全部状态", value: "全部" },
  { label: "LISTENING", value: "LISTENING" },
  { label: "ESTABLISHED", value: "ESTABLISHED" },
  { label: "TIME_WAIT", value: "TIME_WAIT" },
  { label: "CLOSE_WAIT", value: "CLOSE_WAIT" },
  { label: "其他", value: "其他" },
];
</script>

<template>
  <div class="flex shrink-0 items-center gap-3 pt-4">
    <span class="text-xs font-bold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
      筛选
    </span>
    <n-select v-model:value="store.protocol" :options="protocolOptions" class="w-32" size="medium" />
    <n-select v-model:value="store.status" :options="statusOptions" class="w-40" size="medium" />

    <!-- 搜索 + 刷新：放在筛选右侧 -->
    <n-input
      v-model:value="store.search"
      clearable
      size="medium"
      placeholder="搜索进程、端口、PID、软件名…"
      class="ml-auto w-[320px]"
    />
    <n-button
      type="primary"
      size="medium"
      round
      :loading="store.loading"
      @click="store.refresh()"
    >
      刷新
    </n-button>
  </div>
</template>
