<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

// 浏览器开发环境下 Tauri 窗口对象不可用，全部安全兜底
let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
try {
  appWindow = getCurrentWindow();
} catch {
  appWindow = null;
}

async function run(fn: "minimize" | "toggleMaximize" | "close") {
  try {
    await appWindow?.[fn]();
  } catch {
    /* 浏览器环境忽略 */
  }
}
</script>

<template>
  <!-- 整条可拖拽移动窗口（data-tauri-drag-region），交通灯在右侧 -->
  <div
    data-tauri-drag-region
    class="flex h-9 shrink-0 select-none items-center justify-end gap-2 px-4"
  >
    <!-- 最小化（黄） -->
    <button
      class="traffic-light traffic-min"
      title="最小化 / Minimize"
      @click.stop="run('minimize')"
    >
      <svg viewBox="0 0 12 12" aria-hidden="true">
        <path d="M2.5 6h7" stroke="#000" stroke-opacity="0.55" stroke-width="1.6" stroke-linecap="round" />
      </svg>
    </button>
    <!-- 最大化 / 还原（绿） -->
    <button
      class="traffic-light traffic-max"
      title="最大化 / Maximize"
      @click.stop="run('toggleMaximize')"
    >
      <svg viewBox="0 0 12 12" aria-hidden="true">
        <path d="M6 2.5v7M2.5 6h7" stroke="#000" stroke-opacity="0.55" stroke-width="1.6" stroke-linecap="round" />
      </svg>
    </button>
    <!-- 关闭（红） -->
    <button
      class="traffic-light traffic-close"
      title="关闭 / Close"
      @click.stop="run('close')"
    >
      <svg viewBox="0 0 12 12" aria-hidden="true">
        <path
          d="M3 3l6 6M9 3L3 9"
          stroke="#000"
          stroke-opacity="0.55"
          stroke-width="1.6"
          stroke-linecap="round"
        />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.traffic-light {
  width: 13px;
  height: 13px;
  border-radius: 999px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0.5px solid rgba(0, 0, 0, 0.18);
  box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.25);
  cursor: pointer;
  transition: transform 0.1s ease-out;
}
.traffic-light:active {
  transform: scale(0.9);
}
.traffic-light svg {
  width: 8px;
  height: 8px;
  opacity: 0;
  transition: opacity 0.12s ease-out;
}
.traffic-light:hover svg {
  opacity: 1;
}
.traffic-min {
  background: #febc2e;
}
.traffic-max {
  background: #28c840;
}
.traffic-close {
  background: #ff5f57;
}
</style>
