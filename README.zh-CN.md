# PortView

[中文](#-中文) · [English](./README.md)

> 英文版见 **[README.md](./README.md)**。

---

# 🇨🇳 中文

一款轻量的 **Windows** 桌面工具：列出所有 TCP/UDP 端点及其属主进程，并为每个进程展示内存占用与 CPU 占用；支持结束进程、按名称 / 拼音 / PID / 端口模糊搜索、按资源占用排序，整体采用贴纸风格界面。

基于 **Tauri 2 + Vue 3 + TypeScript + Vite + UnoCSS + Pinia + naive-ui** 构建。

> 仓库地址：<https://gitee.com/ShiXiongZhiDao/port-view>

## ✨ 功能特性

- **进程 ↔ 端口合并视图**：列出系统全部进程（含无网络连接的进程，如 Everything），并给出 TCP 数、UDP 数、监听端口数与聚合端口摘要；展开某行可查看每条原始连接（协议 / 本地地址 / 远程地址 / 状态）。
- **进程内存与 CPU**：展示每个进程的物理内存工作集与 CPU 占用率（与任务管理器同口径）。点击 **CPU / 内存** 表头可排序，**默认按 CPU 从大到小**。
- **列宽可拖拽调整**：拖动任意列表头的右边缘即可调整该列宽度，列宽会在下次启动时记住。
- **行右键菜单**：在任意进程行上点击鼠标右键，可**打开程序地址**（在资源管理器中定位该程序）或**结束进程**（带确认）。
- **模糊搜索**：支持进程名、软件名、完整路径、PID、端口以及中文**拼音 / 首字母**匹配（pinyin-pro）。
- **筛选**：按协议（全部 / TCP / UDP）与连接状态（LISTENING / ESTABLISHED / TIME_WAIT / CLOSE_WAIT / 其他）筛选，与搜索叠加生效。
- **手动刷新**：数据为一次性快照，仅在点击"刷新"（或结束进程后）更新，无后台轮询。
- **结束进程**：`taskkill /F`，带气泡二次确认；系统关键进程额外红色警示。
- **亮色 / 暗色 / 跟随系统**主题，并持久化。

## 🧱 技术栈

| 层次 | 选型 |
|---|---|
| 桌面框架 | Tauri 2（Rust） |
| 前端框架 | Vue 3（`<script setup>`） |
| 语言 | TypeScript（strict） |
| 构建 | Vite |
| 样式 | UnoCSS（`preset-wind3`） |
| 状态管理 | Pinia |
| 组件库 | naive-ui |
| 国际化 | 自研轻量 i18n 单例（中 / 英） |
| 拼音搜索 | pinyin-pro |
| 原生能力 | 通过 `windows` crate 调用 iphlpapi、ToolHelp、PSAPI、DWM |

## ✅ 环境要求

- Windows 10 / 11（x64）
- [Node.js](https://nodejs.org/) 与 **pnpm**
- Rust 稳定版工具链（MSVC）以及 **WebView2 运行时**（Win11 自带）
- 运行时需要**管理员权限**——见[为什么需要管理员](#-为什么需要管理员中文)

## 🚀 快速开始

```bash
# 1. 安装前端依赖
pnpm install

# 2a. 仅前端，浏览器中打开（无 Tauri IPC，列表数据为空）
pnpm dev

# 2b. 完整桌面应用——必须在「以管理员身份运行」的终端中执行
pnpm tauri dev
```

> 在非管理员终端运行 `pnpm tauri dev` 会报 `os error 740`，因为程序清单要求 `requireAdministrator`。请右键终端 → **以管理员身份运行**后再执行。

### 生产打包

```bash
pnpm tauri build
```

NSIS 安装包输出到 `target/release/bundle/nsis/`，文件名为 **`PortView_<版本号>_x64-setup.exe`**。安装后，程序（可执行文件、安装目录、开始菜单快捷方式、任务栏 / 托盘图标与卸载器）统一命名为 **PortView**，并使用自定义的绿色 "PV" 应用图标；安装向导、卸载器与程序本体都不会出现 Tauri 默认图标。如需从新的 1024×1024 源图重新生成整套图标，执行 `pnpm tauri icon <源图.png>`（源图必须是真正的 PNG）。

## 🖱️ 使用说明

1. 点击"刷新"获取进程与连接的最新快照。
2. 在搜索框输入名称 / 拼音 / PID / 端口进行过滤；用两个下拉框按协议、状态收窄范围。
3. 点击 **CPU** 或 **内存** 表头切换排序字段与方向（降序 ↔ 升序），**默认按 CPU 从大到小**；拖动任意列表头右边缘可调整列宽。
4. 在进程行上点击鼠标右键，可**打开程序地址**或**结束进程**；点击行内"连接"展开原始端点，点击"结束进程"经确认后终止。
5. 点击齿轮图标打开**设置**，可切换主题 / 语言、打开仓库、检查更新、查看版本信息。


## 🔒 为什么需要管理员？<a id="-为什么需要管理员中文"></a>

可执行文件内嵌 `requireAdministrator` 清单，以便：

- 读取受保护系统进程的完整路径与版本信息；
- 枚举全部 TCP/UDP 表及其属主 PID；
- 使用 `taskkill /F` 结束进程。

非提权状态下程序仍可运行，但部分系统进程路径为空、资源占用显示为 0，且结束这些进程会失败。

## 📁 项目结构

```
port-view/
├─ src/                      # Vue 3 前端
│  ├─ components/            # HeroHeader · FilterBar · ProcessList · TitleBar · SettingsModal
│  ├─ composables/           # useTheme · useUpdate
│  ├─ i18n/                  # 中英字典与 t()
│  ├─ stores/                # Pinia store（合并视图、搜索、筛选、统计）
│  ├─ utils/                 # openExternal()
│  ├─ types.ts · App.vue · style.css
├─ src-tauri/                # Rust 后端
│  ├─ src/
│  │  ├─ lib.rs              # Tauri 命令、系统托盘、窗体圆角
│  │  └─ netinfo.rs          # WinAPI 采集（连接、进程、内存/CPU）
│  ├─ capabilities/          # Tauri 权限
│  └─ tauri.conf.json
├─ public/
│  ├─ fonts/                 # Fredoka / Nunito 本地字体
```


## 📄 许可证

当前尚未添加 LICENSE 文件；在提供之前，著作权归作者所有。
