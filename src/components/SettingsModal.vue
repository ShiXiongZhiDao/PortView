<script setup lang="ts">
import { computed } from "vue";
import { useTheme, type ThemeMode } from "../composables/useTheme";
import { useI18n, type Lang } from "../i18n";
import { openExternal } from "../utils/open";

defineProps<{ show: boolean }>();
const emit = defineEmits<{ (e: "update:show", v: boolean): void }>();

const { themeMode, setTheme } = useTheme();
const { lang, setLang, t } = useI18n();

const themeValue = computed<ThemeMode>({
  get: () => themeMode.value,
  set: (v) => setTheme(v),
});
const langValue = computed<Lang>({
  get: () => lang.value,
  set: (v) => setLang(v),
});

const themeChoices = computed(() => [
  { value: "system", label: t("theme.system"), desc: t("theme.system.desc") },
  { value: "light", label: t("theme.light"), desc: t("theme.light.desc") },
  { value: "dark", label: t("theme.dark"), desc: t("theme.dark.desc") },
]);

const langChoices = computed(() => [
  { value: "zh", label: t("lang.zh") },
  { value: "en", label: t("lang.en") },
]);

const REPO_URL = "https://gitee.com/ShiXiongZhiDao/port-view";
</script>

<template>
  <n-modal :show="show" @update:show="(v: boolean) => emit('update:show', v)">
    <div class="pv-card flex max-h-[84vh] w-[460px] flex-col overflow-hidden">
      <!-- 标题栏 -->
      <div class="flex shrink-0 items-center justify-between px-6 pt-5">
        <h2 class="font-heading text-2xl font-bold" :style="{ color: 'var(--pv-text)' }">
          {{ t("settings.title") }}
        </h2>
        <button
          class="pv-btn flex h-9 w-9 items-center justify-center text-sm font-bold"
          :style="{ background: 'var(--pv-card-2)', color: 'var(--pv-text-soft)' }"
          @click="emit('update:show', false)"
        >
          ✕
        </button>
      </div>

      <div class="overflow-y-auto px-6 pb-6 pt-4">
        <!-- 外观主题 -->
        <div class="flex flex-col gap-2.5">
          <span class="text-xs font-extrabold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
            {{ t("settings.theme") }}
          </span>
          <label
            v-for="c in themeChoices"
            :key="c.value"
            class="flex cursor-pointer items-center gap-3 rounded-xl px-4 py-2.5"
            :style="{
              border: '2px solid var(--pv-stroke)',
              background: themeValue === c.value ? 'var(--pv-green-bg)' : 'var(--pv-card-2)',
            }"
          >
            <input v-model="themeValue" type="radio" :value="c.value" class="h-4 w-4 accent-green-600" />
            <div class="flex flex-col">
              <span class="text-sm font-bold" :style="{ color: 'var(--pv-text)' }">{{ c.label }}</span>
              <span class="text-xs" :style="{ color: 'var(--pv-text-soft)' }">{{ c.desc }}</span>
            </div>
          </label>
        </div>

        <!-- 界面语言 -->
        <div class="mt-5 flex flex-col gap-2.5">
          <span class="text-xs font-extrabold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
            {{ t("settings.language") }}
          </span>
          <div class="flex gap-2.5">
            <label
              v-for="c in langChoices"
              :key="c.value"
              class="pv-press flex flex-1 cursor-pointer items-center justify-center gap-2 rounded-xl px-4 py-2.5 text-sm font-bold"
              :style="{
                border: '2px solid var(--pv-stroke)',
                background: langValue === c.value ? 'var(--pv-blue-bg)' : 'var(--pv-card-2)',
                color: langValue === c.value ? 'var(--pv-blue-text)' : 'var(--pv-text)',
              }"
            >
              <input v-model="langValue" type="radio" :value="c.value" class="h-4 w-4 accent-blue-600" />
              {{ c.label }}
            </label>
          </div>
        </div>

        <!-- 代码仓库 -->
        <div class="mt-5 flex flex-col gap-2.5">
          <span class="text-xs font-extrabold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
            {{ t("settings.repos") }}
          </span>
          <div class="flex gap-2.5">
            <button
              class="pv-btn flex flex-1 items-center justify-center gap-2 px-4 py-2 text-sm font-bold"
              :style="{ background: 'var(--pv-card-2)', color: 'var(--pv-text)' }"
              @click="openExternal(REPO_URL)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.55 0-.27-.01-1.17-.02-2.12-3.2.7-3.88-1.36-3.88-1.36-.52-1.33-1.28-1.68-1.28-1.68-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.69 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.18 1.18a11.1 11.1 0 0 1 5.8 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.84 1.19 3.1 0 4.42-2.69 5.39-5.26 5.68.41.36.78 1.06.78 2.14 0 1.55-.01 2.8-.01 3.18 0 .3.21.67.8.55A11.51 11.51 0 0 0 23.5 12C23.5 5.65 18.35.5 12 .5Z" />
              </svg>
              GitHub
            </button>
            <button
              class="pv-btn flex flex-1 items-center justify-center gap-2 px-4 py-2 text-sm font-bold"
              :style="{ background: 'var(--pv-red-bg, #FEE2E2)', color: '#DC2626' }"
              @click="openExternal(REPO_URL)"
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M11.98 0C5.37 0 0 5.37 0 11.98c0 5.3 3.44 9.8 8.21 11.39.6.11.82-.26.82-.58v-2.03c-3.34.73-4.04-1.61-4.04-1.61-.55-1.39-1.34-1.76-1.34-1.76-1.09-.75.08-.73.08-.73 1.21.09 1.84 1.24 1.84 1.24 1.07 1.84 2.82 1.31 3.51 1 .11-.78.42-1.31.76-1.61-2.67-.3-5.47-1.34-5.47-5.95 0-1.31.47-2.39 1.24-3.23-.13-.3-.54-1.52.12-3.18 0 0 1.01-.32 3.3 1.23a11.5 11.5 0 0 1 6.02 0c2.29-1.55 3.3-1.23 3.3-1.23.66 1.66.25 2.88.12 3.18.77.84 1.24 1.92 1.24 3.23 0 4.62-2.81 5.64-5.49 5.94.43.37.81 1.1.81 2.22v3.29c0 .32.22.7.83.58A12 12 0 0 0 24 11.98C24 5.37 18.63 0 11.98 0Z" />
              </svg>
              Gitee
            </button>
          </div>
        </div>

        <!-- 赞助支持 -->
        <div class="mt-5 flex flex-col gap-2.5">
          <span class="text-xs font-extrabold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
            {{ t("settings.sponsor") }}
          </span>
          <p class="text-xs" :style="{ color: 'var(--pv-text-faint)' }">{{ t("settings.sponsorTip") }}</p>
          <n-image-group>
            <div class="flex gap-3">
              <div
                class="flex flex-col items-center gap-1.5 rounded-xl px-3 py-3"
                :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-card-2)' }"
              >
                <n-image
                  src="/qr/alipay.png"
                  width="96"
                  height="96"
                  object-fit="contain"
                  :show-toolbar="false"
                  :style="{ border: '1.5px solid var(--pv-border)', borderRadius: '8px' }"
                />
                <span class="text-xs font-bold" :style="{ color: '#1677FF' }">{{ t("sponsor.alipay") }}</span>
              </div>
              <div
                class="flex flex-col items-center gap-1.5 rounded-xl px-3 py-3"
                :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-card-2)' }"
              >
                <n-image
                  src="/qr/wechat.png"
                  width="96"
                  height="96"
                  object-fit="contain"
                  :show-toolbar="false"
                  :style="{ border: '1.5px solid var(--pv-border)', borderRadius: '8px' }"
                />
                <span class="text-xs font-bold" :style="{ color: '#16A34A' }">{{ t("sponsor.wechat") }}</span>
              </div>
            </div>
          </n-image-group>
        </div>

        <!-- 关于 -->
        <div
          class="mt-5 flex items-center justify-between border-t pt-4 text-xs"
          :style="{ borderColor: 'var(--pv-border)', color: 'var(--pv-text-faint)' }"
        >
          <span class="font-bold" :style="{ color: 'var(--pv-text-soft)' }">{{ t("settings.about") }}</span>
          <span>Port Process View · v0.1.0</span>
        </div>
      </div>
    </div>
  </n-modal>
</template>
