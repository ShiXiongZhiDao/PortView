<script setup lang="ts">
import { computed } from "vue";
import { useTheme, type ThemeMode } from "../composables/useTheme";

defineProps<{ show: boolean }>();
const emit = defineEmits<{ (e: "update:show", v: boolean): void }>();

const { themeMode, setTheme } = useTheme();
const themeValue = computed<ThemeMode>({
  get: () => themeMode.value,
  set: (v) => setTheme(v),
});

const themeChoices = [
  { value: "system", label: "跟随系统", desc: "自动匹配 Windows 深浅色" },
  { value: "light", label: "亮色", desc: "米白背景 + 白色卡片" },
  { value: "dark", label: "暗色", desc: "深蓝灰背景 + 深色卡片" },
];
</script>

<template>
  <n-modal
    :show="show"
    @update:show="(v: boolean) => emit('update:show', v)"
  >
    <div class="pv-card w-[420px] p-6">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between">
        <h2 class="font-heading text-2xl font-bold" :style="{ color: 'var(--pv-text)' }">
          设置
        </h2>
        <button
          class="pv-btn flex h-9 w-9 items-center justify-center text-sm font-bold"
          :style="{ background: 'var(--pv-card-2)', color: 'var(--pv-text-soft)' }"
          @click="emit('update:show', false)"
        >
          ✕
        </button>
      </div>

      <!-- 主题 -->
      <div class="mt-6 flex flex-col gap-3">
        <span class="text-xs font-extrabold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
          外观主题
        </span>
        <div class="flex flex-col gap-2">
          <label
            v-for="c in themeChoices"
            :key="c.value"
            class="flex cursor-pointer items-center gap-3 rounded-xl px-4 py-3 transition-colors"
            :style="{
              border: '2px solid var(--pv-stroke)',
              background: themeValue === c.value ? 'var(--pv-green-bg)' : 'var(--pv-card-2)',
            }"
          >
            <input
              v-model="themeValue"
              type="radio"
              :value="c.value"
              class="h-4 w-4 accent-green-600"
            />
            <div class="flex flex-col">
              <span class="text-sm font-bold" :style="{ color: 'var(--pv-text)' }">{{ c.label }}</span>
              <span class="text-xs" :style="{ color: 'var(--pv-text-soft)' }">{{ c.desc }}</span>
            </div>
          </label>
        </div>
      </div>

      <!-- 关于 -->
      <div
        class="mt-6 flex items-center justify-between border-t pt-4 text-xs"
        :style="{ borderColor: 'var(--pv-border)', color: 'var(--pv-text-faint)' }"
      >
        <span>port-view · Windows Port &amp; Process Monitor</span>
        <span class="font-bold">v0.1.0</span>
      </div>
    </div>
  </n-modal>
</template>
