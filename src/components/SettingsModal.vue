<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useTheme, type ThemeMode } from "../composables/useTheme";
import { useI18n, type Lang } from "../i18n";
import { useUpdate } from "../composables/useUpdate";
import { openExternal } from "../utils/open";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{ (e: "update:show", v: boolean): void }>();

const { themeMode, setTheme } = useTheme();
const { lang, setLang, t } = useI18n();
const {
  status,
  currentVersion,
  latestVersion,
  releaseNotes,
  releaseUrl,
  checkForUpdates,
  loadCurrentVersion,
} = useUpdate();

const activeTab = ref<"general" | "sponsor" | "about">("general");

// 弹窗每次打开时确保当前版本已加载
watch(
  () => props.show,
  (v) => {
    if (v) loadCurrentVersion();
  }
);

const tabs = computed(() => [
  { id: "general", label: t("settings.tab.general") },
  { id: "sponsor", label: t("settings.tab.sponsor") },
  { id: "about", label: t("settings.tab.about") },
]);

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
    <div class="pv-card w-[460px] p-6">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between">
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

      <!-- Tab 切换 -->
      <div class="mt-4 flex gap-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="pv-btn flex-1 px-1 py-1.5 text-xs"
          :style="{
            background: activeTab === tab.id ? 'var(--pv-green)' : 'var(--pv-card-2)',
            color: activeTab === tab.id ? '#fff' : 'var(--pv-text)',
          }"
          @click="activeTab = tab.id as typeof activeTab"
        >
          {{ tab.label }}
        </button>
      </div>

      <!-- 固定高度内容区：切换 tab 不产生尺寸跳动 -->
      <div class="mt-5 h-[556px] overflow-y-auto pr-1">
        <!-- 通用：主题 + 语言 -->
        <div v-show="activeTab === 'general'" class="flex flex-col gap-5">
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

          <div class="flex flex-col gap-2.5">
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
        </div>

        <!-- 赞助：二维码 + 一毛也是爱 + 关注我们 + 代码仓库 -->
        <div v-show="activeTab === 'sponsor'" class="flex flex-col gap-3">
          <p class="text-xs" :style="{ color: 'var(--pv-text-soft)' }">{{ t("settings.sponsorTip") }}</p>
          <div class="flex justify-center pt-0.5">
            <span
              class="pv-pill text-base"
              :style="{
                background: 'var(--pv-green-bg)',
                color: 'var(--pv-green-text)',
                border: '2px solid var(--pv-green)',
                boxShadow: '2px 2px 0 var(--pv-green)',
                padding: '6px 18px',
              }"
            >
              {{ t("sponsor.coin") }}
            </span>
          </div>
          <n-image-group>
            <div class="grid grid-cols-2 gap-2.5">
              <div
                class="flex flex-col items-center gap-1.5 rounded-xl px-2 py-3"
                :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-card-2)' }"
              >
                <n-image
                  src="/qr/alipay.png"
                  width="100%"
                  object-fit="contain"
                  :show-toolbar="false"
                  :style="{ border: '1.5px solid var(--pv-border)', borderRadius: '8px' }"
                />
                <span class="text-xs font-bold" style="color: #1677ff">{{ t("sponsor.alipay") }}</span>
              </div>
              <div
                class="flex flex-col items-center gap-1.5 rounded-xl px-2 py-3"
                :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-card-2)' }"
              >
                <n-image
                  src="/qr/wechat.png"
                  width="100%"
                  object-fit="contain"
                  :show-toolbar="false"
                  :style="{ border: '1.5px solid var(--pv-border)', borderRadius: '8px' }"
                />
                <span class="text-xs font-bold" style="color: #16a34a">{{ t("sponsor.wechat") }}</span>
              </div>
            </div>
          </n-image-group>

          <!-- 关注我们：微信公众号 / 小程序 -->
          <span class="text-xs font-extrabold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
            {{ t("about.follow") }}
          </span>
          <div class="grid grid-cols-2 gap-2.5">
            <div
              class="flex flex-col items-center gap-1 rounded-xl px-2 py-3"
              :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-card-2)' }"
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="#07C160" aria-hidden="true">
                <path d="M8.691 2.188C3.891 2.188 0 5.476 0 9.53c0 2.212 1.17 4.203 3.002 5.55a.59.59 0 0 1 .213.665l-.39 1.48c-.019.07-.048.141-.048.213 0 .163.13.295.29.295a.326.326 0 0 0 .167-.054l1.903-1.114a.864.864 0 0 1 .717-.098 10.16 10.16 0 0 0 2.837.403c.276 0 .543-.027.811-.05-.857-2.578.157-4.972 1.932-6.446 1.703-1.415 3.882-1.98 5.853-1.838-.576-3.583-4.196-6.348-8.596-6.348zM5.785 5.991c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 0 1-1.162 1.178A1.17 1.17 0 0 1 4.623 7.17c0-.651.52-1.18 1.162-1.18zm5.813 0c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 0 1-1.162 1.178 1.17 1.17 0 0 1-1.162-1.178c0-.651.52-1.18 1.162-1.18zm5.34 2.867c-1.797-.052-3.746.512-5.28 1.786-1.72 1.428-2.687 3.72-1.78 6.22.942 2.453 3.666 4.229 6.884 4.229.826 0 1.622-.12 2.361-.336a.722.722 0 0 1 .598.082l1.584.926a.272.272 0 0 0 .14.047c.134 0 .24-.111.24-.247 0-.06-.023-.12-.038-.177l-.327-1.233a.582.582 0 0 1-.023-.156.49.49 0 0 1 .201-.398C23.024 18.48 24 16.82 24 14.98c0-3.21-2.931-5.837-6.656-6.088V8.89c-.135-.01-.27-.027-.407-.032zm-2.59 3.274c.535 0 .969.44.969.982a.976.976 0 0 1-.969.983.976.976 0 0 1-.969-.983c0-.542.434-.982.97-.982zm4.844 0c.535 0 .969.44.969.982a.976.976 0 0 1-.969.983.976.976 0 0 1-.969-.983c0-.542.434-.982.969-.982z" />
              </svg>
              <span class="text-[10px] font-bold" :style="{ color: 'var(--pv-text-faint)' }">{{ t("about.mp") }}</span>
              <span class="text-sm font-extrabold" :style="{ color: 'var(--pv-text)' }">师兄知道</span>
            </div>
            <div
              class="flex flex-col items-center gap-1 rounded-xl px-2 py-3"
              :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-card-2)' }"
            >
              <svg width="24" height="24" viewBox="0 0 24 24" fill="#07C160" aria-hidden="true">
                <path d="M8.691 2.188C3.891 2.188 0 5.476 0 9.53c0 2.212 1.17 4.203 3.002 5.55a.59.59 0 0 1 .213.665l-.39 1.48c-.019.07-.048.141-.048.213 0 .163.13.295.29.295a.326.326 0 0 0 .167-.054l1.903-1.114a.864.864 0 0 1 .717-.098 10.16 10.16 0 0 0 2.837.403c.276 0 .543-.027.811-.05-.857-2.578.157-4.972 1.932-6.446 1.703-1.415 3.882-1.98 5.853-1.838-.576-3.583-4.196-6.348-8.596-6.348zM5.785 5.991c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 0 1-1.162 1.178A1.17 1.17 0 0 1 4.623 7.17c0-.651.52-1.18 1.162-1.18zm5.813 0c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 0 1-1.162 1.178 1.17 1.17 0 0 1-1.162-1.178c0-.651.52-1.18 1.162-1.18zm5.34 2.867c-1.797-.052-3.746.512-5.28 1.786-1.72 1.428-2.687 3.72-1.78 6.22.942 2.453 3.666 4.229 6.884 4.229.826 0 1.622-.12 2.361-.336a.722.722 0 0 1 .598.082l1.584.926a.272.272 0 0 0 .14.047c.134 0 .24-.111.24-.247 0-.06-.023-.12-.038-.177l-.327-1.233a.582.582 0 0 1-.023-.156.49.49 0 0 1 .201-.398C23.024 18.48 24 16.82 24 14.98c0-3.21-2.931-5.837-6.656-6.088V8.89c-.135-.01-.27-.027-.407-.032zm-2.59 3.274c.535 0 .969.44.969.982a.976.976 0 0 1-.969.983.976.976 0 0 1-.969-.983c0-.542.434-.982.97-.982zm4.844 0c.535 0 .969.44.969.982a.976.976 0 0 1-.969.983.976.976 0 0 1-.969-.983c0-.542.434-.982.969-.982z" />
              </svg>
              <span class="text-[10px] font-bold" :style="{ color: 'var(--pv-text-faint)' }">{{ t("about.miniapp") }}</span>
              <span class="text-sm font-extrabold" :style="{ color: 'var(--pv-text)' }">师兄知道</span>
            </div>
          </div>

          <!-- 代码仓库 -->
          <span class="pt-1 text-xs font-extrabold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
            {{ t("settings.tab.repos") }}
          </span>
          <div class="grid grid-cols-2 gap-2.5">
            <button
              class="pv-btn flex items-center justify-center gap-2 px-3 py-2.5 text-sm font-bold"
              :style="{ background: 'var(--pv-card-2)', color: 'var(--pv-text)' }"
              @click="openExternal(REPO_URL)"
            >
              <svg width="17" height="17" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.11.79-.25.79-.55 0-.27-.01-1.17-.02-2.12-3.2.7-3.88-1.36-3.88-1.36-.52-1.33-1.28-1.68-1.28-1.68-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.69 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.18 1.18a11.1 11.1 0 0 1 5.8 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.84 1.19 3.1 0 4.42-2.69 5.39-5.26 5.68.41.36.78 1.06.78 2.14 0 1.55-.01 2.8-.01 3.18 0 .3.21.67.8.55A11.51 11.51 0 0 0 23.5 12C23.5 5.65 18.35.5 12 .5Z" />
              </svg>
              GitHub
            </button>
            <button
              class="pv-btn flex items-center justify-center gap-2 px-3 py-2.5 text-sm font-bold"
              :style="{ background: '#FEE2E2', color: '#DC2626' }"
              @click="openExternal(REPO_URL)"
            >
              <svg width="17" height="17" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M11.98 0C5.37 0 0 5.37 0 11.98c0 5.3 3.44 9.8 8.21 11.39.6.11.82-.26.82-.58v-2.03c-3.34.73-4.04-1.61-4.04-1.61-.55-1.39-1.34-1.76-1.34-1.76-1.09-.75.08-.73.08-.73 1.21.09 1.84 1.24 1.84 1.24 1.07 1.84 2.82 1.31 3.51 1 .11-.78.42-1.31.76-1.61-2.67-.3-5.47-1.34-5.47-5.95 0-1.31.47-2.39 1.24-3.23-.13-.3-.54-1.52.12-3.18 0 0 1.01-.32 3.3 1.23a11.5 11.5 0 0 1 6.02 0c2.29-1.55 3.3-1.23 3.3-1.23.66 1.66.25 2.88.12 3.18.77.84 1.24 1.92 1.24 3.23 0 4.62-2.81 5.64-5.49 5.94.43.37.81 1.1.81 2.22v3.29c0 .32.22.7.83.58A12 12 0 0 0 24 11.98C24 5.37 18.63 0 11.98 0Z" />
              </svg>
              Gitee
            </button>
          </div>
          <p class="break-all text-center text-[11px]" :style="{ color: 'var(--pv-text-faint)' }">{{ REPO_URL }}</p>
        </div>

        <!-- 关于：标识 + 版本/升级 + 技术栈 -->
        <div v-show="activeTab === 'about'" class="flex flex-col gap-3">
          <div class="flex flex-col items-center gap-1.5 py-1 text-center">
            <div
              class="flex h-14 w-14 items-center justify-center font-heading text-xl font-bold text-white"
              :style="{
                background: 'var(--pv-green)',
                border: '3px solid var(--pv-stroke)',
                borderRadius: '14px',
                boxShadow: '3px 3px 0 var(--pv-stroke)',
              }"
            >
              PV
            </div>
            <div class="font-heading text-lg font-bold" :style="{ color: 'var(--pv-text)' }">PortView</div>
            <div class="text-xs" :style="{ color: 'var(--pv-text-soft)' }">{{ t("about.desc") }}</div>
          </div>

          <!-- 当前版本 -->
          <div
            class="flex items-center justify-between rounded-xl px-4 py-2.5"
            :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-card-2)' }"
          >
            <span class="text-xs font-bold" :style="{ color: 'var(--pv-text-soft)' }">{{ t("about.version") }}</span>
            <span class="font-heading text-sm font-bold" :style="{ color: 'var(--pv-text)' }">v{{ currentVersion }}</span>
          </div>

          <!-- 检查更新 -->
          <button
            class="pv-btn flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-bold text-white"
            :style="{ background: 'var(--pv-green)' }"
            :disabled="status === 'checking'"
            @click="checkForUpdates"
          >
            <svg
              v-if="status === 'checking'"
              class="animate-spin"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              aria-hidden="true"
            >
              <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-opacity="0.3" stroke-width="3" />
              <path d="M21 12a9 9 0 0 0-9-9" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
            </svg>
            {{ status === "checking" ? t("update.checking") : t("update.check") }}
          </button>

          <!-- 已是最新 -->
          <div
            v-if="status === 'latest'"
            class="flex items-center justify-center gap-2 rounded-xl px-4 py-3"
            :style="{ background: 'var(--pv-green-bg)' }"
          >
            <div
              class="flex h-7 w-7 items-center justify-center rounded-full"
              :style="{ background: 'var(--pv-green)' }"
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                <path d="M5 12.5l4.5 4.5L19 7.5" stroke="#fff" stroke-width="2.8" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </div>
            <span class="text-sm font-bold" :style="{ color: 'var(--pv-green-text)' }">{{ t("update.latest") }}</span>
          </div>

          <!-- 发现新版本 -->
          <div
            v-if="status === 'available'"
            class="flex flex-col gap-2.5 rounded-xl p-4"
            :style="{ border: '2px solid var(--pv-stroke)', background: 'var(--pv-yellow-bg)' }"
          >
            <div class="flex items-center gap-2">
              <span class="text-sm font-extrabold" :style="{ color: 'var(--pv-yellow-text)' }">
                {{ t("update.available") }}：v{{ latestVersion }}
              </span>
            </div>
            <template v-if="releaseNotes">
              <span class="text-xs font-bold" :style="{ color: 'var(--pv-text-soft)' }">{{ t("update.notes") }}</span>
              <div
                class="max-h-24 overflow-y-auto whitespace-pre-wrap rounded-lg px-3 py-2 text-xs leading-relaxed"
                :style="{ background: 'var(--pv-card)', color: 'var(--pv-text-soft)', border: '1.5px solid var(--pv-border)' }"
              >
                {{ releaseNotes }}
              </div>
            </template>
            <button
              class="pv-btn mt-0.5 flex items-center justify-center gap-2 px-4 py-2 text-sm font-bold text-white"
              :style="{ background: 'var(--pv-green)' }"
              @click="openExternal(releaseUrl)"
            >
              {{ t("update.download") }}
            </button>
          </div>

          <!-- 检查失败 -->
          <div
            v-if="status === 'error'"
            class="flex items-center justify-center gap-3 rounded-xl px-4 py-3"
            :style="{ background: '#FEF2F2' }"
          >
            <span class="text-xs font-bold" style="color: #DC2626">{{ t("update.error") }}</span>
            <button
              class="pv-btn px-4 py-1.5 text-xs font-bold"
              :style="{ background: '#fff', color: '#DC2626', border: '2px solid #DC2626' }"
              @click="checkForUpdates"
            >
              {{ t("update.retry") }}
            </button>
          </div>

          <div class="text-center text-[11px]" :style="{ color: 'var(--pv-text-faint)' }">{{ t("about.stack") }}</div>
        </div>
      </div>
    </div>
  </n-modal>
</template>
