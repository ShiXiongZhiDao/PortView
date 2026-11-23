/**
 * 更新检查的状态机（见 docs/adr/0002-release-and-upgrade.md）。
 *
 * 只做两件事：问「有没有新版」、把新版交给安装器。Windows 上安装器会先结束本进程，
 * 这是系统限制，界面必须提前说清楚。
 *
 * 状态放在模块级：设置窗口与其他可能出现的地方共用同一份事实，不会各查各的。
 */
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { readonly, ref } from "vue";

export type UpdateStatus =
  | "idle"
  | "checking"
  | "latest"
  | "available"
  | "downloading"
  | "installing"
  | "error";

// 浏览器预览 / 取 Tauri 版本失败时的兜底；发版时由 scripts/release.mjs 同步。
const FALLBACK_VERSION = "0.2.0";
const RELEASES_PAGE = "https://gitee.com/ShiXiongZhiDao/PortView/releases";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const status = ref<UpdateStatus>("idle");
const currentVersion = ref(FALLBACK_VERSION);
const latestVersion = ref("");
const releaseNotes = ref("");
const releaseUrl = ref(RELEASES_PAGE);
const progress = ref(0);
const installed = ref<boolean | null>(null);
/** check() 的返回对象只在本次会话有效，下载必须复用同一份。 */
let pending: Update | null = null;
let versionLoaded = false;
let autoCheckScheduled = false;

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

const isBusy = () =>
  status.value === "checking" || status.value === "downloading" || status.value === "installing";

async function loadCurrentVersion() {
  if (versionLoaded) return;
  versionLoaded = true;
  try {
    currentVersion.value = await getVersion();
  } catch {
    currentVersion.value = FALLBACK_VERSION;
  }
}

async function loadInstallKind(): Promise<boolean> {
  if (!inTauri) return false;
  if (installed.value !== null) return installed.value;
  // 便携版可以检查更新，但不能就地替换自身；失败时按便携版处理，宁可少给一个按钮。
  installed.value = await invoke<boolean>("is_installed_build").catch(() => false);
  return installed.value;
}

/**
 * `silent`：设置页打开后的自动检查。失败就当没查过 —— 离线或 DNS 抽风
 * 不该在用户没主动问的时候留一条红色报错。
 */
async function checkForUpdates(options: { silent?: boolean } = {}) {
  if (!inTauri || isBusy()) return;
  await loadCurrentVersion();
  await loadInstallKind();

  status.value = "checking";
  try {
    const update = await check({ timeout: options.silent ? 15000 : 30000 });
    if (!update) {
      pending = null;
      status.value = "latest";
      return;
    }
    pending = update;
    latestVersion.value = update.version.replace(/^[vV]/, "");
    releaseNotes.value = update.body?.trim() ?? "";
    status.value = "available";
  } catch (error) {
    pending = null;
    status.value = options.silent ? "idle" : "error";
    if (!options.silent) releaseNotes.value = messageOf(error);
  }
}

/** 设置页出现 5 秒后再查：先把界面渲染完，别和启动抢带宽。 */
function scheduleAutoCheck(delayMs = 5000) {
  if (!inTauri || autoCheckScheduled) return;
  autoCheckScheduled = true;
  window.setTimeout(() => {
    // 用户在这 5 秒里已经手动查过，就不要覆盖他的结果。
    if (status.value === "idle") void checkForUpdates({ silent: true });
  }, delayMs);
}

async function installUpdate() {
  if (!pending || status.value !== "available") return;
  if (!(await loadInstallKind())) return;

  const update = pending;
  try {
    status.value = "downloading";
    progress.value = 0;
    let downloaded = 0;
    let total = 0;
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength ?? 0;
          break;
        case "Progress":
          downloaded += event.data.chunkLength;
          if (total > 0) progress.value = Math.min(99, Math.floor((downloaded / total) * 100));
          break;
        case "Finished":
          break;
      }
    });
    status.value = "installing";
    // Windows 上安装器会先结束本进程，这行通常走不到；
    // 留着是为了「用户取消安装器」时还能把界面拉回来。
    await relaunch().catch(() => undefined);
  } catch (error) {
    status.value = "error";
    releaseNotes.value = messageOf(error);
  }
}

export function useUpdate() {
  return {
    status: readonly(status),
    currentVersion: readonly(currentVersion),
    latestVersion: readonly(latestVersion),
    releaseNotes: readonly(releaseNotes),
    releaseUrl: readonly(releaseUrl),
    progress: readonly(progress),
    installed: readonly(installed),
    loadCurrentVersion,
    loadInstallKind,
    checkForUpdates,
    scheduleAutoCheck,
    installUpdate,
  };
}
