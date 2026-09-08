import { ref, computed, watch } from "vue";
import { useOsTheme } from "naive-ui";

export type ThemeMode = "system" | "dark" | "light";

/** 模块级单例：所有组件共享同一份主题状态 */
const themeMode = ref<ThemeMode>(
  (localStorage.getItem("port-view-theme") as ThemeMode) || "system",
);
const osTheme = useOsTheme();
const isDark = computed(() =>
  themeMode.value === "system" ? osTheme.value === "dark" : themeMode.value === "dark",
);

// 同步到 <html data-theme>，保证 teleport 到 body 的弹窗（n-modal 等）也能继承 CSS 变量
watch(
  isDark,
  (dark) => {
    document.documentElement.setAttribute("data-theme", dark ? "dark" : "light");
  },
  { immediate: true },
);

export function useTheme() {
  function setTheme(v: ThemeMode) {
    themeMode.value = v;
    localStorage.setItem("port-view-theme", v);
  }
  return { themeMode, isDark, setTheme };
}
