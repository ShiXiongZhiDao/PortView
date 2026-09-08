import { openUrl } from "@tauri-apps/plugin-opener";

/** 在系统默认浏览器打开外部链接；浏览器开发环境回退 window.open */
export async function openExternal(url: string): Promise<void> {
  try {
    await openUrl(url);
  } catch {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}
