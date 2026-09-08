<script setup lang="ts">
import { h, ref } from "vue";
import {
  NButton,
  NPopconfirm,
  useMessage,
  type DataTableColumns,
} from "naive-ui";
import { useConnectionsStore } from "../stores/connections";
import type { ProcessInfo } from "../types";
import ExpandedRow from "./ExpandedRow.vue";

const store = useConnectionsStore();
const message = useMessage();

// 展开行：记录已展开的 PID
const expandedPids = ref<number[]>([]);

async function handleKill(row: ProcessInfo) {
  try {
    const result = await store.kill(row.pid);
    message.success(result);
  } catch (e) {
    message.error(String(e));
  } finally {
    await store.refresh();
  }
}

function isCritical(row: ProcessInfo): boolean {
  const name = row.name.toLowerCase();
  return (
    row.pid <= 4 ||
    name === "system" ||
    name === "svchost.exe" ||
    name === "csrss.exe" ||
    name === "wininit.exe" ||
    name === "services.exe" ||
    name === "lsass.exe" ||
    name === "explorer.exe"
  );
}

/** 端口摘要：最多显示 6 个，超出折叠 */
function renderPorts(row: ProcessInfo & { _ports: number[] }) {
  const ports = row._ports;
  if (!ports.length) return h("span", { class: "text-xs opacity-50" }, "-");
  const show = ports.slice(0, 6);
  const more = ports.length - show.length;
  const content = show.join(", ") + (more > 0 ? ` +${more}` : "");
  return h(
    "span",
    { class: "tabular-nums", title: ports.join(", ") },
    content,
  );
}

const columns: DataTableColumns<ProcessInfo & { _ports: number[] }> = [
  {
    title: "进程",
    key: "name",
    minWidth: 280,
    sorter: (a, b) => a.name.localeCompare(b.name),
    render: (row) =>
      h("div", { class: "flex flex-col" }, [
        h("span", { class: "font-medium" }, row.name),
        row.path
          ? h(
              "span",
              { class: "truncate text-xs opacity-50", style: "max-width: 340px" },
              row.path,
            )
          : null,
      ]),
  },
  { title: "PID", key: "pid", width: 80, sorter: (a, b) => a.pid - b.pid },
  {
    title: "端口",
    key: "_ports",
    width: 160,
    render: (row) => renderPorts(row),
  },
  {
    title: "TCP",
    key: "tcp",
    width: 70,
    align: "right",
    sorter: (a, b) => a.tcp - b.tcp,
    render: (row) => (row.tcp > 0 ? row.tcp : "-"),
  },
  {
    title: "UDP",
    key: "udp",
    width: 70,
    align: "right",
    sorter: (a, b) => a.udp - b.udp,
    render: (row) => (row.udp > 0 ? row.udp : "-"),
  },
  {
    title: "监听端口",
    key: "listening",
    width: 90,
    align: "right",
    sorter: (a, b) => a.listening - b.listening,
    render: (row) => (row.listening > 0 ? row.listening : "-"),
  },
  {
    title: "软件名",
    key: "software",
    width: 200,
    ellipsis: { tooltip: true },
    render: (row) => row.software || "-",
  },
  {
    title: "操作",
    key: "actions",
    width: 110,
    align: "center",
    render: (row) =>
      h(
        NPopconfirm,
        {
          positiveText: "结束",
          negativeText: "取消",
          onPositiveClick: () => handleKill(row),
        },
        {
          trigger: () =>
            h(
              NButton,
              { type: "error", size: "small", secondary: true, round: true },
              { default: () => "结束进程" },
            ),
          default: () =>
            isCritical(row)
              ? `危险操作：结束 ${row.name}（PID ${row.pid}）可能导致系统异常，确认结束？`
              : `确认结束进程 ${row.name}（PID ${row.pid}）？`,
        },
      ),
  },
];
</script>

<template>
  <n-data-table
    :columns="columns"
    :data="store.filteredProcesses"
    :loading="store.loading"
    :bordered="false"
    :max-height="`calc(100vh - 320px)`"
    :row-key="(row: ProcessInfo) => row.pid"
    :expanded-row-keys="expandedPids"
    :scroll-x="1180"
    size="small"
    @update:expanded-row-keys="expandedPids = $event"
  >
    <template #expanded-row="{ row }">
      <ExpandedRow :row="row as ProcessInfo" />
    </template>
  </n-data-table>
</template>
