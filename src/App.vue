<script setup lang="ts">
import { computed, onMounted } from "vue";
import { darkTheme } from "naive-ui";
import { useConnectionsStore } from "./stores/connections";
import { useTheme } from "./composables/useTheme";
import HeroHeader from "./components/HeroHeader.vue";
import FilterBar from "./components/FilterBar.vue";
import ProcessList from "./components/ProcessList.vue";
import TitleBar from "./components/TitleBar.vue";

const store = useConnectionsStore();
const { isDark } = useTheme();
const theme = computed(() => (isDark.value ? darkTheme : null));

/** LearnHub 风格 naive-ui 主题覆盖（亮/暗） */
const themeOverrides = computed(() =>
  isDark.value
    ? {
        common: {
          primaryColor: "#22C55E",
          primaryColorHover: "#4ADE80",
          primaryColorPressed: "#16A34A",
          primaryColorSuppl: "#22C55E",
          successColor: "#22C55E",
          errorColor: "#F87171",
          warningColor: "#FBBF24",
          infoColor: "#60A5FA",
          bodyColor: "transparent",
          cardColor: "#1E293B",
          modalColor: "#1E293B",
          popoverColor: "#1E293B",
          tableColor: "transparent",
          tableHeaderColor: "#1A2337",
          textColorBase: "#F1F5F9",
          textColor1: "#F1F5F9",
          textColor2: "#CBD5E1",
          textColor3: "#94A3B8",
          borderColor: "#2B3648",
          dividerColor: "#263041",
          actionColor: "#172033",
          inputColor: "#1E293B",
          inputColorDisabled: "#182136",
          hoverColor: "rgba(34,197,94,0.10)",
          pressedColor: "rgba(34,197,94,0.18)",
          borderRadius: "10px",
          borderRadiusSmall: "8px",
          borderRadiusMedium: "10px",
          borderRadiusLarge: "14px",
          fontFamily:
            "'Nunito', 'Segoe UI', 'Microsoft YaHei', system-ui, sans-serif",
          fontWeightStrong: "800",
        },
        Button: {
          borderRadiusMedium: "999px",
          borderRadiusSmall: "999px",
          fontWeight: "800",
        },
        Input: { borderRadius: "10px" },
        Select: { peers: { InternalSelection: { borderRadius: "10px" } } },
        Tag: { borderRadius: "999px" },
        Popover: { borderRadius: "14px" },
      }
    : {
        common: {
          primaryColor: "#16A34A",
          primaryColorHover: "#22C55E",
          primaryColorPressed: "#15803D",
          primaryColorSuppl: "#22C55E",
          successColor: "#16A34A",
          errorColor: "#DC2626",
          warningColor: "#D97706",
          infoColor: "#2563EB",
          bodyColor: "transparent",
          cardColor: "#FFFFFF",
          modalColor: "#FFFFFF",
          popoverColor: "#FFFFFF",
          tableColor: "transparent",
          tableHeaderColor: "#F8F9FB",
          textColorBase: "#1F2937",
          textColor1: "#1F2937",
          textColor2: "#4B5563",
          textColor3: "#6B7280",
          borderColor: "#EEF0F3",
          dividerColor: "#EEF0F3",
          actionColor: "#F8F9FB",
          inputColor: "#FFFFFF",
          inputColorDisabled: "#F3F4F6",
          hoverColor: "rgba(22,163,74,0.07)",
          pressedColor: "rgba(22,163,74,0.14)",
          borderRadius: "10px",
          borderRadiusSmall: "8px",
          borderRadiusMedium: "10px",
          borderRadiusLarge: "14px",
          fontFamily:
            "'Nunito', 'Segoe UI', 'Microsoft YaHei', system-ui, sans-serif",
          fontWeightStrong: "800",
        },
        Button: {
          borderRadiusMedium: "999px",
          borderRadiusSmall: "999px",
          fontWeight: "800",
        },
        Input: { borderRadius: "10px" },
        Select: { peers: { InternalSelection: { borderRadius: "10px" } } },
        Tag: { borderRadius: "999px" },
        Popover: { borderRadius: "14px" },
      },
);

onMounted(() => {
  store.refresh();
});
</script>

<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-global-style />
    <n-message-provider placement="top-right">
      <div
        class="flex h-screen flex-col overflow-hidden rounded-[14px]"
        :data-theme="isDark ? 'dark' : 'light'"
        :style="{ background: 'var(--pv-bg)' }"
      >
        <!-- 自定义标题栏：可拖拽 + 右侧交通灯按钮 -->
        <TitleBar />

        <div class="flex min-h-0 flex-1 flex-col px-7 pb-6">
          <!-- Hero 标题区（logo + 大标题 + 统计块 + 设置） -->
          <HeroHeader />

          <!-- 筛选条 -->
          <FilterBar />

          <!-- 进程列表 -->
          <div class="min-h-0 flex-1 pt-4">
            <ProcessList />
          </div>
        </div>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>
