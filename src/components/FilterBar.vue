<script setup lang="ts">
import { computed } from "vue";
import { useConnectionsStore } from "../stores/connections";
import { useI18n } from "../i18n";

const store = useConnectionsStore();
const { t } = useI18n();

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
</script>

<template>
  <div class="flex shrink-0 items-center gap-3 pt-4">
    <span class="text-xs font-bold tracking-wider" :style="{ color: 'var(--pv-text-soft)' }">
      {{ t("filter.label") }}
    </span>
    <n-select v-model:value="store.protocol" :options="protocolOptions" class="w-32" size="medium" />
    <n-select v-model:value="store.status" :options="statusOptions" class="w-40" size="medium" />

    <!-- 搜索 + 刷新：放在筛选右侧 -->
    <n-input
      v-model:value="store.search"
      clearable
      size="medium"
      :placeholder="t('search.placeholder')"
      class="ml-auto w-[320px] pv-search"
    />
    <n-button type="primary" size="medium" round :loading="store.loading" @click="store.refresh()">
      {{ t("btn.refresh") }}
    </n-button>
  </div>
</template>
