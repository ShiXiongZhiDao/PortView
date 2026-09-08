<script setup lang="ts">
import { computed } from "vue";
import { NTag } from "naive-ui";
import { useConnectionsStore } from "../stores/connections";
import type { ProcessInfo } from "../types";

const props = defineProps<{ row: ProcessInfo }>();
const store = useConnectionsStore();

const conns = computed(() => store.connectionsOf(props.row.pid));
</script>

<template>
  <div class="px-4 py-3">
    <div class="mb-2 text-xs font-medium opacity-70">
      网络连接（{{ conns.length }} 条）— PID {{ row.pid }}
    </div>
    <table v-if="conns.length" class="w-full border-collapse text-xs tabular-nums">
      <thead>
        <tr class="opacity-60">
          <th class="px-2 py-1 text-left font-medium">协议</th>
          <th class="px-2 py-1 text-left font-medium">本地端口</th>
          <th class="px-2 py-1 text-left font-medium">本地地址</th>
          <th class="px-2 py-1 text-left font-medium">远程地址</th>
          <th class="px-2 py-1 text-left font-medium">状态</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="c in conns"
          :key="`${c.protocol}-${c.localPort}-${c.remoteAddress}-${c.state}`"
          class="border-t border-dashed opacity-90"
        >
          <td class="px-2 py-1">{{ c.protocol }}</td>
          <td class="px-2 py-1">{{ c.localPort }}</td>
          <td class="px-2 py-1">{{ c.localAddress }}:{{ c.localPort }}</td>
          <td class="px-2 py-1">{{ c.remoteAddress }}</td>
          <td class="px-2 py-1">
            <n-tag v-if="c.state === 'LISTENING'" size="small" type="success" :bordered="false">
              {{ c.state }}
            </n-tag>
            <template v-else>{{ c.state }}</template>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-else class="text-xs opacity-60">该进程当前没有网络连接</div>
  </div>
</template>
