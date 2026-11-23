<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { darkTheme } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useConnectionsStore } from "./stores/connections";
import { useTheme } from "./composables/useTheme";
import { useAutoRefresh } from "./composables/useAutoRefresh";
import { useShortcuts } from "./composables/useShortcuts";
import { useI18n } from "./i18n";
import { trackAppStarted } from "./lib/analytics";
import { decodeElevationState, type LaunchState } from "./lib/launch-state";
import HeroHeader from "./components/HeroHeader.vue";
import FilterBar from "./components/FilterBar.vue";
import ProcessList from "./components/ProcessList.vue";
import PortList from "./components/PortList.vue";
import TitleBar from "./components/TitleBar.vue";

const store = useConnectionsStore();
const { isDark, themeMode } = useTheme();
const { lang, t, syncLanguageToBackend } = useI18n();
const theme = computed(() => (isDark.value ? darkTheme : null));

/** 部分采集失败的明细（ADR-0009：告警不能被静默吞掉，两个视图都要看得到） */
const warningDetail = computed(() =>
  store.warnings
    .map((w) => `${w.source}${w.addressFamily ? `/${w.addressFamily}` : ""}: ${w.message}`)
    .join(" · "),
);

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

// 定时刷新单例：在应用根部启动一次，timer 生命周期与 App 一致（ADR-0007）。
const autoRefresh = useAutoRefresh(store);
// FilterBar 组件实例供 Ctrl+F 聚焦搜索框；弹层内的搜索不在此列。
const filterBarRef = ref<{ focusSearch: () => void } | null>(null);
useShortcuts(store, () => filterBarRef.value?.focusSearch());

onMounted(async () => {
  // 托盘菜单语言要跟界面一致（系统语言默认值也在这里同步给 Rust）。
  syncLanguageToBackend();
  // 启动配置快照只取一次；统计失败被 analytics 自己吞掉，不影响首屏刷新。
  trackAppStarted(themeMode.value, lang.value);

  // 提权接管：恢复旧实例的视图与筛选，再抓第一份快照（ADR-0010）。
  try {
    const launch = await invoke<LaunchState>("get_launch_state");
    store.isElevated = launch.isElevated;
    const restored = decodeElevationState(launch.state);
    if (restored) {
      if (restored.view) store.view = restored.view;
      if (restored.search !== undefined) store.search = restored.search;
      if (restored.protocol) store.protocol = restored.protocol;
      if (restored.status) store.status = restored.status;
      if (restored.family) store.family = restored.family;
      if (restored.listeningOnly !== undefined) store.listeningOnly = restored.listeningOnly;
    }
  } catch {
    // 旧版本没有该命令时按普通启动处理，不影响首屏。
  }

  await store.refreshSnapshot();
});

onUnmounted(() => autoRefresh.stop());
</script>

<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-global-style />
    <!-- 通知放顶部居中：右上角会遮住无边框窗口的交通灯按钮 -->
    <n-message-provider placement="top">
      <n-dialog-provider>
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

          <!-- 视图切换 + 筛选条：合并为一行 -->
          <div class="mt-3 flex min-w-0 shrink-0 items-center gap-2.5">
            <div
              class="flex shrink-0 gap-1 rounded-xl p-1"
              :style="{
                background: 'var(--pv-card-2)',
                border: '2px solid var(--pv-stroke)',
                boxShadow: '2px 2px 0 var(--pv-stroke)',
              }"
            >
              <button
                v-for="v in (['process', 'port'] as const)"
                :key="v"
                class="rounded-lg px-4 py-1 text-xs font-extrabold transition-colors"
                :style="store.view === v
                  ? { background: 'var(--pv-green)', color: '#fff', border: '2px solid var(--pv-stroke)', boxShadow: '2px 2px 0 var(--pv-stroke)' }
                  : { background: 'transparent', color: 'var(--pv-text-soft)', border: '2px solid transparent' }"
                @click="store.view = v"
              >
                {{ v === 'process' ? t('view.process') : t('view.port') }}
              </button>
            </div>

            <FilterBar ref="filterBarRef" />
          </div>

          <!-- 部分采集失败：不遮断列表，但也不能静默吞掉（ADR-0009） -->
          <div
            v-if="store.warnings.length"
            class="mt-2 flex min-w-0 shrink-0 items-center gap-2 rounded-lg px-3 py-1 text-[11px]"
            :style="{
              background: 'var(--pv-amber-bg)',
              color: 'var(--pv-amber-text)',
              border: '1.5px solid var(--pv-amber-text)',
            }"
            :title="warningDetail"
          >
            <span class="shrink-0 font-extrabold">
              {{ t("warning.partial", { count: store.warnings.length }) }}
            </span>
            <span class="truncate opacity-80">{{ warningDetail }}</span>
          </div>

          <!-- 当前视图主体 -->
          <div class="min-h-0 flex-1 pt-3">
            <ProcessList v-if="store.view === 'process'" />
            <PortList v-else />
          </div>
        </div>
      </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
