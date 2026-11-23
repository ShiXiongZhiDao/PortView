<script setup lang="ts">
import { computed, ref } from "vue";
import { useConnectionsStore } from "../stores/connections";
import { useAutoRefresh } from "../composables/useAutoRefresh";
import { useI18n } from "../i18n";

const store = useConnectionsStore();
const { t } = useI18n();
const auto = useAutoRefresh(store);
const searchInputEl = ref<{ focus: () => void } | null>(null);
defineExpose({ focusSearch: () => searchInputEl.value?.focus() });

const protocolOptions = computed(() => [
  { label: t("filter.allProtocol"), value: "全部" },
  { label: "TCP", value: "TCP" },
  { label: "UDP", value: "UDP" },
]);

const statusOptions = computed(() => [
  { label: t("filter.allStatus"), value: "全部" },
  { label: "LISTENING", value: "LISTENING" },
  { label: "ESTABLISHED", value: "ESTABLISHED" },
  { label: "TIME_WAIT", value: "TIME_WAIT" },
  { label: "CLOSE_WAIT", value: "CLOSE_WAIT" },
  { label: t("filter.allStatus") === "全部状态" ? "其他" : "Other", value: "其他" },
]);

const familyOptions = computed(() => [
  { label: t("filter.allFamily"), value: "全部" },
  { label: "IPv4", value: "IPv4" },
  { label: "IPv6", value: "IPv6" },
]);
</script>

<template>
  <div class="flex min-w-0 flex-1 items-center gap-2">
    <n-select v-model:value="store.protocol" :options="protocolOptions" class="w-28" size="medium" :disabled="store.listeningOnly" />
    <n-select v-model:value="store.status" :options="statusOptions" class="w-36" size="medium" :disabled="store.listeningOnly" />
    <n-select v-model:value="store.family" :options="familyOptions" class="w-24" size="medium" />
    <n-button
      size="medium"
      :type="store.listeningOnly ? 'primary' : 'default'"
      @click="store.listeningOnly = !store.listeningOnly"
    >
      {{ t("filter.listeningOnly") }}
    </n-button>

    <!-- 搜索 + 刷新：放在筛选右侧；输入条件后回车即拉取最新数据 -->
    <n-input
      ref="searchInputEl"
      v-model:value="store.search"
      clearable
      size="medium"
      :placeholder="t('search.placeholder')"
      class="ml-auto w-[260px] pv-search"
      @keyup.enter="store.refreshSnapshot()"
    />
    <!-- 定时刷新（ADR-0007）：顶栏只负责开关，间隔在设置页 -->
    <div class="flex shrink-0 items-center gap-1.5" :title="t('refresh.tip')">
      <n-switch :value="auto.enabled.value" @update:value="auto.setEnabled" />
      <span class="text-xs font-bold" :style="{ color: 'var(--pv-text-soft)' }">{{ t("refresh.auto") }}</span>
    </div>
    <n-button type="primary" size="medium" round :loading="store.loading" @click="store.refreshSnapshot()">
      {{ t("btn.refresh") }}
    </n-button>
  </div>
</template>
