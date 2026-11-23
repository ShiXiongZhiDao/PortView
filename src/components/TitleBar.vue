<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";

// 浏览器开发环境下 Tauri 窗口对象不可用，全部安全兜底
let appWindow: ReturnType<typeof getCurrentWindow> | null = null;
try {
  appWindow = getCurrentWindow();
  // 关闭窗口 → 最小化到系统托盘（真正退出走托盘"退出"菜单）
  appWindow.onCloseRequested(async (event) => {
    event.preventDefault();
    try {
      await appWindow?.hide();
    } catch {
      /* 忽略 */
    }
  });
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
    class="flex h-9 shrink-0 select-none items-center justify-end gap-1 px-4"
  >
    <!-- 最小化（黄）：外层加大热区，内层保留 14px 圆点 -->
    <button class="traffic-hit" title="最小化 / Minimize" @click.stop="run('minimize')">
      <span class="traffic-light traffic-min">
        <svg viewBox="0 0 12 12" aria-hidden="true">
          <path d="M2.5 6h7" stroke="#000" stroke-opacity="0.55" stroke-width="1.6" stroke-linecap="round" />
        </svg>
      </span>
    </button>
    <!-- 最大化 / 还原（绿） -->
    <button class="traffic-hit" title="最大化 / Maximize" @click.stop="run('toggleMaximize')">
      <span class="traffic-light traffic-max">
        <svg viewBox="0 0 12 12" aria-hidden="true">
          <path d="M6 2.5v7M2.5 6h7" stroke="#000" stroke-opacity="0.55" stroke-width="1.6" stroke-linecap="round" />
        </svg>
      </span>
    </button>
    <!-- 关闭（红） -->
    <button class="traffic-hit" title="关闭 / Close" @click.stop="run('close')">
      <span class="traffic-light traffic-close">
        <svg viewBox="0 0 12 12" aria-hidden="true">
          <path
            d="M3 3l6 6M9 3L3 9"
            stroke="#000"
            stroke-opacity="0.55"
            stroke-width="1.6"
            stroke-linecap="round"
          />
        </svg>
      </span>
    </button>
  </div>
</template>

<style scoped>
/* 热区 36×30 的透明按钮：视觉圆点仍是 14px，点击范围明显加大加长 */
.traffic-hit {
  display: grid;
  place-items: center;
  width: 36px;
  height: 30px;
  padding: 0;
  border: 0;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
}
.traffic-hit:hover {
  background: rgba(128, 128, 128, 0.12);
}
.traffic-hit:focus-visible {
  outline: 2px solid var(--pv-green);
  outline-offset: -2px;
}
.traffic-light {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0.5px solid rgba(0, 0, 0, 0.18);
  box-shadow: inset 0 0 0 0.5px rgba(255, 255, 255, 0.25);
  transition: transform 0.1s ease-out;
}
.traffic-hit:active .traffic-light {
  transform: scale(0.9);
}
.traffic-light svg {
  width: 8px;
  height: 8px;
  opacity: 0;
  transition: opacity 0.12s ease-out;
}
.traffic-hit:hover .traffic-light svg {
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
