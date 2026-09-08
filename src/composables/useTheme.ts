import { ref, computed } from "vue";
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

export function useTheme() {
  function setTheme(v: ThemeMode) {
    themeMode.value = v;
    localStorage.setItem("port-view-theme", v);
  }
  return { themeMode, isDark, setTheme };
}
