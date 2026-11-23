/**
 * 复制文本到剪贴板并给出统一提示。
 * 用 navigator.clipboard（WebView2 基于 Chromium，安全上下文可用）；失败时明确报错，
 * 不静默吞掉——复制失败却弹"成功"是最糟的一类体验。
 */
import { useMessage } from "naive-ui";
import { useI18n } from "../i18n";

/** 双击行会连带触发两次单击（如双击进程名会复制两次），
 *  400ms 内相同文本的提示只弹一次，避免 toast 刷屏；复制动作仍照常执行。 */
const TOAST_DEDUPE_MS = 400;
let lastToast = "";
let lastToastAt = 0;

export function useClipboard() {
  const message = useMessage();
  const { t } = useI18n();

  function toastOnce(text: string, label: string) {
    const now = Date.now();
    const key = `${label}:${text}`;
    if (key === lastToast && now - lastToastAt < TOAST_DEDUPE_MS) return;
    lastToast = key;
    lastToastAt = now;
    message.success(t("toast.copied", { label }));
  }

  async function copy(text: string, label: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(text);
      toastOnce(text, label);
    } catch {
      message.error(t("toast.copyFailed"));
    }
  }

  return { copy };
}
