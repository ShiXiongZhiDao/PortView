<script setup lang="ts">
import { h } from "vue";
import {
  NButton,
  NPopconfirm,
  NTag,
  useMessage,
  type DataTableColumns,
} from "naive-ui";
import { useConnectionsStore } from "../stores/connections";
import type { ConnectionInfo } from "../types";

const store = useConnectionsStore();
const message = useMessage();

/** 结束进程：调用后端 taskkill，完成后自动刷新 */
async function handleKill(row: ConnectionInfo) {
  try {
    const result = await store.kill(row.pid);
    message.success(result);
  } catch (e) {
    message.error(String(e));
  } finally {
    await store.refresh();
  }
}

function isCritical(row: ConnectionInfo): boolean {
  const name = row.processName.toLowerCase();
  return name === "system" || name === "svchost.exe" || row.pid === 4;
}

const columns: DataTableColumns<ConnectionInfo> = [
  { title: "协议", key: "protocol", width: 80 },
  {
    title: "本地端口",
    key: "localPort",
    width: 100,
    sorter: (a, b) => a.localPort - b.localPort,
  },
  { title: "本地地址", key: "localAddress", width: 190, ellipsis: { tooltip: true } },
  { title: "远程地址", key: "remoteAddress", width: 190, ellipsis: { tooltip: true } },
  {
    title: "状态",
    key: "state",
    width: 120,
    sorter: (a, b) => a.state.localeCompare(b.state),
    render: (row) =>
      h(
        NTag,
        {
          type: row.state === "LISTENING" ? "success" : "default",
          size: "small",
          bordered: false,
        },
        { default: () => row.state },
      ),
  },
  {
    title: "进程",
    key: "processName",
    minWidth: 300,
    render: (row) =>
      h("div", { class: "flex flex-col" }, [
        h("span", { class: "font-medium" }, row.processName),
        row.processPath
          ? h(
              "span",
              { class: "truncate text-xs opacity-50", style: "max-width: 340px" },
              row.processPath,
            )
          : null,
      ]),
  },
  { title: "PID", key: "pid", width: 90, sorter: (a, b) => a.pid - b.pid },
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
              ? `危险操作：结束 ${row.processName}（PID ${row.pid}）可能导致系统异常，确认结束？`
              : `确认结束进程 ${row.processName}（PID ${row.pid}）？`,
        },
      ),
  },
];
</script>

<template>
  <n-data-table
    :columns="columns"
    :data="store.filteredRows"
    :loading="store.loading"
    :bordered="false"
    :max-height="`calc(100vh - 320px)`"
    :row-key="(_row: ConnectionInfo, index: number) => index"
    :scroll-x="1120"
    size="small"
  />
</template>
