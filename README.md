# Port Process View

[English](#-english) · [中文](#-中文)

A lightweight **Windows** desktop tool that lists every TCP/UDP endpoint, the process that owns each port, and — for every process — its memory footprint and CPU usage. Kill a process, fuzzy-search by name / pinyin / PID / port, and sort by resource usage, all in one sticker-style interface.

Built with **Tauri 2 + Vue 3 + TypeScript + Vite + UnoCSS + Pinia + naive-ui**.

> Repository: <https://gitee.com/ShiXiongZhiDao/port-view>

---

# 🇬🇧 English

## ✨ Features

- **Process ↔ port unified view** — every running process is listed (including processes with no network connection, e.g. Everything), together with its TCP count, UDP count, listening ports and an aggregated port summary. Expand a row to inspect every raw connection (protocol / local address / remote address / state).
- **Per-process memory & CPU** — physical working set and CPU percentage (same convention as Task Manager) for each process. Click the **CPU / Memory** column header to sort; results are sorted by memory **descending by default**.
- **Fuzzy search** — match by process name, software name, full path, PID, port, and Chinese **pinyin / initials** (powered by `pinyin-pro`).
- **Filters** — filter by protocol (All / TCP / UDP) and by connection state (LISTENING / ESTABLISHED / TIME_WAIT / CLOSE_WAIT / Other), stacked with the search query.
- **Manual refresh** — data is a snapshot taken only when you press **Refresh** (or after killing a process); there is no background polling.
- **Terminate process** — `taskkill /F` with a confirmation popover; critical system processes get an extra red warning.
- **Bilingual UI** — switch between 简体中文 and English in Settings; the choice is persisted.
- **Light / Dark / System theme**, persisted.
- **Frameless custom window** — macOS-style "traffic-light" controls (minimize / maximize / close) on the right, a draggable title region, and rounded window corners.
- **Check for updates** — queries the Gitee Releases API, compares semantic versions, and links to the new release.
- **Settings tabs** — General / Sponsor / About; version info, update check and the repository links all live in **About**, and the dialog keeps a fixed size to avoid layout jumps.

## 🧱 Tech Stack

| Layer | Choice |
|---|---|
| Desktop shell | Tauri 2 (Rust) |
| UI framework | Vue 3 (`<script setup>`) |
| Language | TypeScript (strict) |
| Bundler | Vite |
| Styling | UnoCSS (`preset-wind3`) |
| State | Pinia |
| Components | naive-ui |
| Localization | Custom lightweight i18n singleton (zh / en) |
| Pinyin search | pinyin-pro |
| Native APIs | Windows `iphlpapi`, ToolHelp, PSAPI, DWM via the `windows` crate |

## ✅ Requirements

- Windows 10 / 11 (x64)
- [Node.js](https://nodejs.org/) + **pnpm**
- Rust toolchain (stable, MSVC) and the **WebView2 Runtime** (preinstalled on Windows 11)
- **Administrator rights** at runtime — see [Why administrator](#-why-administrator-english)

## 🚀 Getting Started

```bash
# 1. Install frontend dependencies
pnpm install

# 2a. Frontend only, in a browser (no Tauri IPC, lists will be empty)
pnpm dev

# 2b. Full desktop app — MUST run from an elevated (Administrator) shell
pnpm tauri dev
```

> Launching `pnpm tauri dev` from a non-elevated terminal fails with `os error 740`, because the app manifest requests `requireAdministrator`. Right-click your terminal → **Run as administrator**, then run the command.

### Production build

```bash
pnpm tauri build
```

## 🖱️ Usage

1. Press **Refresh** to take a fresh snapshot of processes and connections.
2. Type in the search box to filter by name / pinyin / PID / port; use the two dropdowns to narrow by protocol and state.
3. Click the **CPU** or **Memory** header to change sort field and direction (desc ↔ asc).
4. Click **Connections** on a row to expand its raw endpoints; click **Kill** to terminate it after confirmation.
5. Open **Settings** via the gear icon to change theme/language, open the repository, check for updates, or view version info.

### Window controls

The native title bar is removed. Drag the window from the thin strip at the very top; use the three traffic-light buttons on the upper-right — yellow = minimize, green = maximize/restore, red = close.

## 🔒 Why administrator? <a id="-why-administrator-english"></a>

The executable embeds a `requireAdministrator` manifest so it can:

- read the full image path / version info of protected system processes;
- enumerate all TCP/UDP tables and their owning PIDs;
- terminate processes with `taskkill /F`.

Without elevation the app still runs, but some system processes show blank paths and zero resource usage, and killing them will fail.

## 📁 Project Structure

```
port-view/
├─ src/                      # Vue 3 frontend
│  ├─ components/            # HeroHeader · FilterBar · ProcessList · TitleBar · SettingsModal
│  ├─ composables/           # useTheme · useUpdate
│  ├─ i18n/                  # zh/en dictionaries + t()
│  ├─ stores/                # Pinia store (merge view, search, filter, stats)
│  ├─ utils/                 # openExternal()
│  ├─ types.ts · App.vue · style.css
├─ src-tauri/                # Rust backend
│  ├─ src/
│  │  ├─ lib.rs              # Tauri commands, window rounded corners
│  │  └─ netinfo.rs          # WinAPI collection (connections, processes, memory/CPU)
│  ├─ capabilities/          # Tauri permissions
│  └─ tauri.conf.json
├─ public/
│  ├─ fonts/                 # Fredoka / Nunito (self-hosted)
│  └─ qr/                    # Sponsor QR codes
└─ docs/                     # ADRs, glossary and design-system tokens
```

## 📐 Design & Architecture Notes

Design decisions are recorded as Architecture Decision Records under [`docs/`](docs/):

- **ADR-001** Tech stack & project layout
- **ADR-002** Network/process data collection (WinAPI)
- **ADR-003** Process termination & privilege model
- **ADR-004** Frontend interaction model (layout, i18n, theme, settings, window)
- **ADR-005** Per-process memory/CPU collection & sorting

## 📄 License

No license file has been added yet. All rights reserved by the author until one is provided.

---
---

# 🇨🇳 中文

一款轻量的 **Windows** 桌面工具：列出所有 TCP/UDP 端点及其属主进程，并为每个进程展示内存占用与 CPU 占用；支持结束进程、按名称 / 拼音 / PID / 端口模糊搜索、按资源占用排序，整体采用贴纸风格界面。

基于 **Tauri 2 + Vue 3 + TypeScript + Vite + UnoCSS + Pinia + naive-ui** 构建。

> 仓库地址：<https://gitee.com/ShiXiongZhiDao/port-view>

## ✨ 功能特性

- **进程 ↔ 端口合并视图**：列出系统全部进程（含无网络连接的进程，如 Everything），并给出 TCP 数、UDP 数、监听端口数与聚合端口摘要；展开某行可查看每条原始连接（协议 / 本地地址 / 远程地址 / 状态）。
- **进程内存与 CPU**：展示每个进程的物理内存工作集与 CPU 占用率（与任务管理器同口径）。点击 **CPU / 内存** 表头可排序，**默认按内存从大到小**。
- **模糊搜索**：支持进程名、软件名、完整路径、PID、端口以及中文**拼音 / 首字母**匹配（pinyin-pro）。
- **筛选**：按协议（全部 / TCP / UDP）与连接状态（LISTENING / ESTABLISHED / TIME_WAIT / CLOSE_WAIT / 其他）筛选，与搜索叠加生效。
- **手动刷新**：数据为一次性快照，仅在点击"刷新"（或结束进程后）更新，无后台轮询。
- **结束进程**：`taskkill /F`，带气泡二次确认；系统关键进程额外红色警示。
- **中英双语**：设置中切换简体中文 / English，选择会被记住。
- **亮色 / 暗色 / 跟随系统**主题，并持久化。
- **无边框自定义窗体**：右侧 macOS 风格"交通灯"按钮（最小化 / 最大化 / 关闭）、可拖拽标题区、窗体圆角。
- **检查更新**：请求 Gitee Releases API，做语义版本对比，并提供新版本下载入口。
- **设置页 Tab 化**：通用 / 赞助 / 关于三个 Tab；版本、检查更新与代码仓库入口统一收在**关于**页，弹窗尺寸固定不跳动。

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

## 🖱️ 使用说明

1. 点击"刷新"获取进程与连接的最新快照。
2. 在搜索框输入名称 / 拼音 / PID / 端口进行过滤；用两个下拉框按协议、状态收窄范围。
3. 点击 **CPU** 或 **内存** 表头切换排序字段与方向（降序 ↔ 升序）。
4. 点击行内"连接"展开原始端点；点击"结束进程"经确认后终止。
5. 点击齿轮图标打开**设置**，可切换主题 / 语言、打开仓库、检查更新、查看版本信息。

### 窗口控制

程序已移除原生标题栏。可在窗口最顶部的细条区域拖拽移动窗口；右上角三个交通灯按钮：黄色 = 最小化，绿色 = 最大化 / 还原，红色 = 关闭。

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
│  │  ├─ lib.rs              # Tauri 命令、窗体圆角
│  │  └─ netinfo.rs          # WinAPI 采集（连接、进程、内存/CPU）
│  ├─ capabilities/          # Tauri 权限
│  └─ tauri.conf.json
├─ public/
│  ├─ fonts/                 # Fredoka / Nunito 本地字体
│  └─ qr/                    # 赞助二维码
└─ docs/                     # ADR、术语表与设计系统 token
```

## 📐 设计与架构说明

关键设计决策以架构决策记录（ADR）形式保存在 [`docs/`](docs/) 目录：

- **ADR-001** 技术栈与项目结构
- **ADR-002** 网络 / 进程数据采集（WinAPI）
- **ADR-003** 进程终止与权限模型
- **ADR-004** 前端交互模型（布局、i18n、主题、设置、窗体）
- **ADR-005** 进程内存 / CPU 采集与排序

## 📄 许可证

当前尚未添加 LICENSE 文件；在提供之前，著作权归作者所有。
