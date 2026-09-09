<script setup lang="ts">
import { computed, ref } from "vue";
import { useConnectionsStore } from "../stores/connections";
import { useI18n } from "../i18n";
import SettingsModal from "./SettingsModal.vue";

const store = useConnectionsStore();
const { t } = useI18n();
const showSettings = ref(false);

/** Hero 右侧统计块（彩色贴纸小块） */
const stats = computed(() => [
  { label: t("stat.tcp"), value: store.stats.tcp, bg: "var(--pv-green-bg)", text: "var(--pv-green-text)" },
  { label: t("stat.udp"), value: store.stats.udp, bg: "var(--pv-blue-bg)", text: "var(--pv-blue-text)" },
  { label: t("stat.listening"), value: store.stats.listening, bg: "var(--pv-amber-bg)", text: "var(--pv-amber-text)" },
  { label: t("stat.processes"), value: store.stats.procs, bg: "var(--pv-violet-bg)", text: "var(--pv-violet-text)" },
]);
</script>

<template>
  <div class="pv-card flex shrink-0 items-center justify-between gap-6 px-6 py-3.5">
    <!-- 左侧：品牌 -->
    <div class="flex items-center gap-3">
      <div
        class="flex h-12 w-12 shrink-0 items-center justify-center font-heading text-lg font-bold text-white"
        :style="{
          background: 'var(--pv-green)',
          border: '3px solid var(--pv-stroke)',
          borderRadius: '12px',
          boxShadow: '3px 3px 0 var(--pv-stroke)',
        }"
      >
        PV
      </div>
      <div class="flex flex-col gap-1">
        <h1 class="font-heading text-2xl font-bold leading-none" :style="{ color: 'var(--pv-text)' }">
          PortView
        </h1>
        <p class="text-xs" :style="{ color: 'var(--pv-text-soft)' }">
          {{ t("app.desc") }}
        </p>
      </div>
    </div>

    <!-- 右侧：统计块 + 设置图标 -->
    <div class="flex shrink-0 items-center gap-3">
      <div
        v-for="s in stats"
        :key="s.label"
        class="flex flex-col items-center gap-0.5 rounded-xl px-3 py-1.5"
        :style="{ background: s.bg, border: '2px solid var(--pv-stroke)', minWidth: '60px' }"
      >
        <span class="font-heading text-xl font-bold leading-none tabular-nums" :style="{ color: s.text }">
          {{ s.value }}
        </span>
        <span class="text-[11px] font-extrabold tracking-wider" :style="{ color: s.text }">
          {{ s.label }}
        </span>
      </div>

      <!-- 设置图标按钮 -->
      <button
        class="pv-btn ml-2 flex h-10 w-10 items-center justify-center"
        :style="{ background: 'var(--pv-card-2)', color: 'var(--pv-text)' }"
        :title="t('btn.settings')"
        @click="showSettings = true"
      >
        <svg width="18" height="18" viewBox="0 0 256 256" fill="currentColor" aria-hidden="true">
          <path
            d="M128 80a48 48 0 1 0 48 48 48.05 48.05 0 0 0-48-48Zm0 76a28 28 0 1 1 28-28 28 28 0 0 1-28 28Zm88.55-28c0-3.18-.14-6.32-.4-9.42l23.06-18a8 8 0 0 0 1.94-10.18l-21.8-37.76a8 8 0 0 0-9.7-3.52l-27.2 10.92a84.3 84.3 0 0 0-16.3-9.42l-4.14-29A8 8 0 0 0 154 16h-43.66a8 8 0 0 0-7.95 7.06l-4.14 29a84.3 84.3 0 0 0-16.3 9.42L50.6 41.64a8 8 0 0 0-9.7 3.52L19.1 82.92a8 8 0 0 0 1.94 10.18l23.06 18a88.7 88.7 0 0 0 0 18.84l-23.06 18a8 8 0 0 0-1.94 10.18l21.8 37.76a8 8 0 0 0 9.7 3.52l27.2-10.92a84.3 84.3 0 0 0 16.3 9.42l4.14 29A8 8 0 0 0 110.34 240h43.32a8 8 0 0 0 7.95-7.06l4.14-29a84.3 84.3 0 0 0 16.3-9.42l27.2 10.92a8 8 0 0 0 9.7-3.52l21.8-37.76a8 8 0 0 0-1.94-10.18l-23.06-18c.26-3.1.4-6.24.4-9.42Z"
          />
        </svg>
      </button>
    </div>

    <!-- 设置弹窗 -->
    <SettingsModal v-model:show="showSettings" />
  </div>
</template>
