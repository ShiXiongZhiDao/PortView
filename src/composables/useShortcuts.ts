/**
 * 全局快捷键（设计包 §6）：Ctrl+F 聚焦搜索、F5 刷新。
 * Esc 不清空搜索，也不在这里拦截——弹层由 naive-ui 自身处理 Esc 关闭。
 */
import { onMounted, onUnmounted } from "vue";
import type { useConnectionsStore } from "../stores/connections";

type Store = ReturnType<typeof useConnectionsStore>;

export function useShortcuts(store: Store, focusSearch: () => void) {
  function onKeydown(e: KeyboardEvent) {
    // Ctrl+F：聚焦搜索框，不触发浏览器查找
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      focusSearch();
      return;
    }
    // F5：刷新快照，避免浏览器/页面默认刷新
    if (e.key === "F5") {
      e.preventDefault();
      void store.refreshSnapshot();
    }
  }

  onMounted(() => window.addEventListener("keydown", onKeydown));
  onUnmounted(() => window.removeEventListener("keydown", onKeydown));
}
