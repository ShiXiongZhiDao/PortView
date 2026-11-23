/**
 * 匿名使用统计（ADR-0003）。
 *
 * 只在启动时上报两件事：应用被启动、启动时的主题与语言配置。
 * 没有用户 ID、设备 ID、使用时长，也不上报任何自由文本或运行期行为。
 *
 * 插件本身不自动采集事件、也没有运行时开关；所有事件都从这里手动发出。
 * 一律 fire-and-forget：统计失败绝不能影响用户操作，但失败会在控制台留痕。
 */
import { invoke } from "@tauri-apps/api/core";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** 事件名集中在此；删事件时保留常量标 @deprecated，不要在别处写裸字符串。 */
export const Events = {
  AppStarted: "app_started",
  ConfigurationSnapshot: "configuration_snapshot",
} as const;

export type AnalyticsProps = Record<string, string | number>;

/** 属性只允许白名单里的枚举值；任何别的值都被清洗掉，防止自由文本混进看板。 */
const ALLOWED: Record<string, ReadonlySet<string | number>> = {
  theme: new Set(["light", "dark", "system"]),
  language: new Set(["zh", "en"]),
};

function sanitize(props: AnalyticsProps): AnalyticsProps {
  const out: AnalyticsProps = {};
  for (const [key, value] of Object.entries(props)) {
    const allowed = ALLOWED[key];
    out[key] = allowed && allowed.has(value) ? value : "[redacted]";
  }
  return out;
}

function track(name: string, props?: AnalyticsProps): void {
  if (!inTauri) return;
  // 不 await、不 throw：统计管线出问题不该在界面上暴露，也不该阻断任何操作。
  void invoke("plugin:aptabase|track_event", {
    name,
    props: props ? sanitize(props) : null,
  }).catch((error) => {
    console.warn("[analytics] 上报失败", name, error);
  });
}

/**
 * 启动事件去重。localStorage 跨窗口共享但会跨进程保留，所以叠一个 5 分钟窗：
 * 窗内视为同一次启动，窗外视为新进程。存储不可用（隐私模式）时退化成每次都报——
 * 宁可多算，不要一条都不发。
 */
const DEDUPE_WINDOW_MS = 5 * 60 * 1000;
const START_STAMP_KEY = "portview.analytics.startedAt";

function claimOnce(key: string): boolean {
  try {
    const last = Number(localStorage.getItem(key) ?? 0);
    const now = Date.now();
    if (now - last < DEDUPE_WINDOW_MS) return false;
    localStorage.setItem(key, String(now));
    return true;
  } catch {
    return true;
  }
}

export function trackAppStarted(theme: string, language: string): void {
  if (!claimOnce(START_STAMP_KEY)) return;
  track(Events.AppStarted);
  // 配置快照随启动上报一次：默认配置也计入，反映「用户在用什么」。
  // 运行期切换不单独上报，下次启动才反映到快照里。
  track(Events.ConfigurationSnapshot, { theme, language });
}
