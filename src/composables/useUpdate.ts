import { ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";

// 兜底版本（浏览器环境 / 取 Tauri 版本失败时）
const FALLBACK_VERSION = "0.1.0";
const LATEST_API =
  "https://gitee.com/api/v5/repos/ShiXiongZhiDao/port-view/releases/latest";
const RELEASES_PAGE = "https://gitee.com/ShiXiongZhiDao/port-view/releases";

export type UpdateStatus = "idle" | "checking" | "latest" | "available" | "error";

// 模块级单例，跨组件共享同一份检查状态
const status = ref<UpdateStatus>("idle");
const currentVersion = ref(FALLBACK_VERSION);
const latestVersion = ref("");
const releaseNotes = ref("");
const releaseUrl = ref(RELEASES_PAGE);
let loaded = false;

/** 解析语义版本号为可比较的数字数组（去掉 v 前缀，忽略非数字后缀） */
function parseVersion(v: string): number[] {
  return v
    .replace(/^[vV]/, "")
    .split(/[.-]/)
    .map((seg) => Number.parseInt(seg, 10) || 0);
}

/** latest 是否比 current 新 */
function isNewer(latest: string, current: string): boolean {
  const a = parseVersion(latest);
  const b = parseVersion(current);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const x = a[i] ?? 0;
    const y = b[i] ?? 0;
    if (x > y) return true;
    if (x < y) return false;
  }
  return false;
}

async function loadCurrentVersion() {
  if (loaded) return;
  loaded = true;
  try {
    currentVersion.value = await getVersion();
  } catch {
    currentVersion.value = FALLBACK_VERSION;
  }
}

async function checkForUpdates() {
  status.value = "checking";
  try {
    await loadCurrentVersion();
    const controller = new AbortController();
    const timer = window.setTimeout(() => controller.abort(), 10000);
    const res = await fetch(LATEST_API, {
      signal: controller.signal,
      headers: { Accept: "application/json" },
    });
    window.clearTimeout(timer);
    if (!res.ok) {
      // 尚未发布任何 Release（404）视为已是最新版本；其余状态码才算失败
      if (res.status === 404) {
        latestVersion.value = "";
        releaseNotes.value = "";
        status.value = "latest";
        return;
      }
      throw new Error(`HTTP ${res.status}`);
    }
    const data: Record<string, unknown> = await res.json();
    latestVersion.value = String(data.tag_name ?? "").replace(/^[vV]/, "");
    releaseNotes.value = String(data.body ?? "").trim();
    releaseUrl.value = String(data.html_url ?? RELEASES_PAGE) || RELEASES_PAGE;
    status.value = isNewer(latestVersion.value, currentVersion.value)
      ? "available"
      : "latest";
  } catch {
    status.value = "error";
  }
}

export function useUpdate() {
  return {
    status,
    currentVersion,
    latestVersion,
    releaseNotes,
    releaseUrl,
    checkForUpdates,
    loadCurrentVersion,
  };
}
