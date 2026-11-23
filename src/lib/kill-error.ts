/**
 * 结束进程失败提示（ADR-0008）。
 * 后端只返回结构化错误码，文案由前端按当前语言渲染，未知码保留原文便于排查。
 */

type Translate = (key: string, vars?: Record<string, string | number>) => string;

/** 后端错误可能是字符串、Error 或带 :detail 的码；统一取第一段错误码。 */
export function errorCode(raw: unknown): string {
  const value =
    typeof raw === "string"
      ? raw.trim()
      : raw instanceof Error
        ? raw.message.trim()
        : String(raw ?? "").trim();
  return value.split(":")[0] || "UNKNOWN";
}

export function killErrorMessage(t: Translate, raw: unknown): string {
  const code = errorCode(raw);
  const key = `kill.err.${code}`;
  const text = t(key);
  return text === key ? t("kill.err.unknown", { code }) : text;
}
