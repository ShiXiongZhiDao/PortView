import { ref, computed, watch } from "vue";
import { useOsTheme } from "naive-ui";
import { trackFeatureToggled } from "../lib/analytics";

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
    // 只记布尔语义（是否显式选了亮/暗），不记具体值以外的自由文本（ADR-0003）。
    trackFeatureToggled("theme", v !== "system");
  }
  return { themeMode, isDark, setTheme };
}
