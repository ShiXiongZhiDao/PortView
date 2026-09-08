<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { darkTheme, useOsTheme, useThemeVars } from "naive-ui";
import { useConnectionsStore } from "./stores/connections";
import StatCards from "./components/StatCards.vue";
import Toolbar from "./components/Toolbar.vue";
import ProcessesTable from "./components/ProcessesTable.vue";

const store = useConnectionsStore();
const osTheme = useOsTheme();
const themeVars = useThemeVars();

type ThemeMode = "system" | "dark" | "light";
const themeMode = ref<ThemeMode>(
  (localStorage.getItem("port-view-theme") as ThemeMode) || "system",
);
const isDark = computed(() =>
  themeMode.value === "system" ? osTheme.value === "dark" : themeMode.value === "dark",
);
const theme = computed(() => (isDark.value ? darkTheme : null));
watch(themeMode, (v) => localStorage.setItem("port-view-theme", v));

const themeOptions = [
  { label: "跟随系统", value: "system" },
  { label: "暗色", value: "dark" },
  { label: "亮色", value: "light" },
];

onMounted(() => {
  store.refresh();
});
</script>

<template>
  <n-config-provider :theme="theme">
    <n-global-style />
    <n-message-provider placement="top-right">
      <div
        class="flex h-screen flex-col overflow-hidden px-6 pb-4"
        :style="{ background: themeVars.bodyColor }"
      >
        <!-- 标题栏 -->
        <header class="flex h-16 shrink-0 items-center justify-between">
          <div class="flex flex-col leading-tight">
            <span
              class="text-[10px] uppercase tracking-[0.25em]"
              :style="{ color: themeVars.textColor3 }"
            >
              Windows Port Watcher
            </span>
            <span class="text-xl font-bold" :style="{ color: themeVars.textColor1 }">
              port-view
            </span>
          </div>
          <div class="flex items-center gap-3">
            <n-select v-model:value="themeMode" :options="themeOptions" class="w-28" size="small" />
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
        </header>

        <!-- 统计卡片 -->
        <StatCards />

        <!-- 搜索 / 筛选 -->
        <Toolbar />

        <!-- 数据表格：进程 + 连接合并视图 -->
        <div class="min-h-0 flex-1 pt-3">
          <ProcessesTable />
        </div>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>
