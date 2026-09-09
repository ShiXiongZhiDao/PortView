import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export type Lang = "zh" | "en";

/** 模块级单例语言状态，持久化到 localStorage */
const lang = ref<Lang>((localStorage.getItem("port-view-lang") as Lang) || "zh");

type Dict = Record<string, string>;

const zh: Dict = {
  "app.desc": "Windows 端口与进程监控 · 实时掌握网络连接",
  "stat.tcp": "TCP",
  "stat.udp": "UDP",
  "stat.listening": "监听",
  "stat.processes": "进程",
  "filter.label": "筛选",
  "filter.allProtocol": "全部协议",
  "filter.allStatus": "全部状态",
  "search.placeholder": "搜索进程、端口、PID、软件名、拼音…",
  "btn.refresh": "刷新",
  "btn.settings": "设置",
  "list.col.process": "进程",
  "list.col.pid": "PID",
  "list.col.cpu": "CPU",
  "list.col.memory": "内存",
  "list.col.tcp": "TCP",
  "list.col.udp": "UDP",
  "list.col.listening": "监听",
  "list.col.ports": "端口",
  "list.col.status": "状态",
  "list.col.actions": "操作",
  "status.active": "活跃",
  "status.idle": "空闲",
  "btn.connections": "连接",
  "btn.collapse": "收起",
  "btn.kill": "结束",
  "btn.cancel": "取消",
  "confirm.critical": "危险操作：结束 {name}（PID {pid}）可能导致系统异常，确认结束？",
  "confirm.kill": "确认结束进程 {name}（PID {pid}）？",
  "empty.title": "没有匹配的进程",
  "empty.tip": "试试调整搜索关键词或筛选条件",
  "ctx.openLocation": "打开程序地址",
  "ctx.noPath": "无法获取程序路径",
  "settings.title": "设置",
  "settings.tab.general": "通用",
  "settings.tab.repos": "仓库",
  "settings.tab.update": "升级",
  "settings.tab.sponsor": "赞助",
  "settings.tab.about": "关于",
  "settings.theme": "外观主题",
  "theme.system": "跟随系统",
  "theme.system.desc": "自动匹配 Windows 深浅色",
  "theme.light": "亮色",
  "theme.light.desc": "纯白背景 + 白色卡片",
  "theme.dark": "暗色",
  "theme.dark.desc": "深蓝灰背景 + 深色卡片",
  "settings.language": "界面语言",
  "lang.zh": "简体中文",
  "lang.en": "English",
  "settings.about": "关于",
  "settings.repos": "代码仓库",
  "settings.sponsor": "赞助支持",
  "settings.sponsorTip": "如果这个工具对你有帮助，请作者喝杯咖啡",
  "sponsor.coin": "一毛也是爱",
  "sponsor.alipay": "支付宝",
  "sponsor.wechat": "微信",
  "about.version": "版本",
  "about.desc": "Windows 端口与进程实时监控工具",
  "about.stack": "技术栈：Tauri 2 · Vue 3 · TypeScript · naive-ui",
  "about.follow": "关注我们",
  "about.mp": "微信公众号",
  "about.miniapp": "微信小程序",
  "update.current": "当前版本",
  "update.check": "检查更新",
  "update.checking": "正在检查更新…",
  "update.latest": "已是最新版本",
  "update.available": "发现新版本",
  "update.notes": "更新说明",
  "update.download": "下载最新版",
  "update.retry": "重试",
  "update.error": "检查失败，请检查网络后重试",
  "update.idle": "点击下方按钮检查是否有新版本",
  "settings.close": "关闭",
};

const en: Dict = {
  "app.desc": "Windows Port & Process Monitor · real-time network connections",
  "stat.tcp": "TCP",
  "stat.udp": "UDP",
  "stat.listening": "Listen",
  "stat.processes": "Procs",
  "filter.label": "Filter",
  "filter.allProtocol": "All protocols",
  "filter.allStatus": "All states",
  "search.placeholder": "Search process, port, PID, app name, pinyin…",
  "btn.refresh": "Refresh",
  "btn.settings": "Settings",
  "list.col.process": "Process",
  "list.col.pid": "PID",
  "list.col.cpu": "CPU",
  "list.col.memory": "Memory",
  "list.col.tcp": "TCP",
  "list.col.udp": "UDP",
  "list.col.listening": "Listen",
  "list.col.ports": "Ports",
  "list.col.status": "Status",
  "list.col.actions": "Actions",
  "status.active": "Active",
  "status.idle": "Idle",
  "btn.connections": "Links",
  "btn.collapse": "Hide",
  "btn.kill": "Kill",
  "btn.cancel": "Cancel",
  "confirm.critical":
    "Warning: killing {name} (PID {pid}) may destabilize the system. Continue?",
  "confirm.kill": "Kill process {name} (PID {pid})?",
  "empty.title": "No matching process",
  "empty.tip": "Try adjusting the keyword or filters",
  "ctx.openLocation": "Open file location",
  "ctx.noPath": "Program path unavailable",
  "settings.title": "Settings",
  "settings.tab.general": "General",
  "settings.tab.repos": "Repos",
  "settings.tab.update": "Update",
  "settings.tab.sponsor": "Sponsor",
  "settings.tab.about": "About",
  "settings.theme": "Appearance",
  "theme.system": "System",
  "theme.system.desc": "Follow Windows light/dark mode",
  "theme.light": "Light",
  "theme.light.desc": "Pure white background + white cards",
  "theme.dark": "Dark",
  "theme.dark.desc": "Dark blue-gray background + dark cards",
  "settings.language": "Language",
  "lang.zh": "简体中文",
  "lang.en": "English",
  "settings.about": "About",
  "settings.repos": "Repositories",
  "settings.sponsor": "Sponsor",
  "settings.sponsorTip": "If this tool helps, buy me a coffee",
  "sponsor.coin": "Every bit counts",
  "sponsor.alipay": "Alipay",
  "sponsor.wechat": "WeChat",
  "about.version": "Version",
  "about.desc": "Real-time Windows port & process monitor",
  "about.stack": "Built with Tauri 2 · Vue 3 · TypeScript · naive-ui",
  "about.follow": "Follow Us",
  "about.mp": "Official Account",
  "about.miniapp": "Mini Program",
  "update.current": "Current version",
  "update.check": "Check for updates",
  "update.checking": "Checking for updates…",
  "update.latest": "You're up to date",
  "update.available": "Update available",
  "update.notes": "Release notes",
  "update.download": "Download latest",
  "update.retry": "Retry",
  "update.error": "Check failed, please verify your network and retry",
  "update.idle": "Click the button below to check for updates",
  "settings.close": "Close",
};

export function useI18n() {
  function setLang(l: Lang) {
    lang.value = l;
    localStorage.setItem("port-view-lang", l);
    // 同步系统托盘菜单语言（Rust 侧 set_language 重建菜单）
    invoke("set_language", { lang: l }).catch(() => {});
  }
  /** 翻译函数：渲染期调用会收集 lang 依赖，切换语言自动重渲染 */
  function t(key: string, vars?: Record<string, string | number>): string {
    let s = lang.value === "en" ? en[key] ?? zh[key] ?? key : zh[key] ?? key;
    if (vars) {
      Object.entries(vars).forEach(([k, v]) => {
        s = s.replace(`{${k}}`, String(v));
      });
    }
    return s;
  }
  return { lang, setLang, t };
}
