# AGENTS.md — port-view

> 给 AI 编码代理 / 开发者快速理解并安全修改本仓库的指南。
> 设计决策的完整来龙去脉见 `docs/ADR-*.md`；术语口径见 `docs/GLOSSARY.md`；视觉 token 见 `docs/design-system/MASTER.md`。
> 本文件描述的是**现状事实**，若与代码不一致，以代码为准并顺手修正本文。

## 1. 项目概览

`port-view`（对外展示名 **Port Process View**）是一款 **Windows 专属** 的轻量桌面工具：

- 列出系统全部 TCP / UDP 端点及其属主进程；
- 为每个进程展示内存工作集与 CPU 占用（任务管理器口径）；
- 支持按名称 / 拼音 / PID / 端口 / 路径模糊搜索、按协议与状态筛选、按 CPU / 内存排序（**默认 CPU 降序**）；
- 所有列可拖拽调整宽度（持久化）；
- 行**右键菜单**：打开程序地址 / 结束进程；
- 支持 `taskkill /F` 结束进程（带确认与系统关键进程警示）；
- **系统托盘**：关闭窗口即隐藏到托盘；托盘右键菜单可「显示 / 退出」，左键单击托盘切换显隐，菜单文案随界面中英切换同步。

**硬约束：仅支持 Windows（x64，Win10/11）。** 运行时必须以管理员权限启动。`src-tauri/` 里的 Rust 代码直接调用 WinAPI，无跨平台目标；`netinfo.rs` 中的 `#[cfg(test)]` 单元测试同样只在 Windows 上成立。

## 2. 技术栈

| 层次 | 选型 | 备注 |
|---|---|---|
| 桌面框架 | Tauri 2（Rust） | 命令注册在 `src-tauri/src/lib.rs` |
| 前端框架 | Vue 3（`<script setup>` SFC） | |
| 语言 | TypeScript（strict） | `tsconfig.json` 开启 `noUnusedLocals` 等 |
| 构建 | Vite 8 | 固定端口 `1420`（`vite.config.ts`） |
| 样式 | UnoCSS（`preset-wind3`）+ 全局 CSS 变量 | 贴纸风格 token 见 `src/style.css` |
| 状态管理 | Pinia | `src/stores/connections.ts` |
| 组件库 | naive-ui | 主题覆盖在 `src/App.vue` |
| 国际化 | 自研 i18n 单例（zh / en） | `src/i18n/index.ts` |
| 拼音搜索 | pinyin-pro | 搜索索引在 store 内预计算 |
| 原生能力 | `windows` crate（0.62） | iphlpapi、ToolHelp、PSAPI、DWM |

## 3. 常用命令

在仓库根目录执行（`pnpm` 是唯一包管理器）：

| 命令 | 用途 | 注意 |
|---|---|---|
| `pnpm install` | 安装前端依赖 | |
| `pnpm dev` | 仅前端（浏览器） | **无 Tauri IPC**，列表为空属正常 |
| `pnpm tauri dev` | 完整桌面应用 | **必须管理员终端**，否则报 `os error 740` |
| `pnpm build` | 前端类型检查 + 打包到 `dist/` | `vue-tsc --noEmit && vite build` |
| `pnpm tauri build` | 生产打包（NSIS/MSI 等） | 打包本身无需提权；运行产物需管理员 |
| `cargo test` | 运行 Rust 单元测试 | 在 `src-tauri/` 目录下执行 |

> 非管理员终端运行 `pnpm tauri dev` 会失败，因为 `build.rs` 内嵌了 `requireAdministrator` 清单。请右键终端 →「以管理员身份运行」。

## 4. 项目结构

```
port-view/
├─ AGENTS.md                  # 本文件
├─ README.md                  # 用户说明（英文版）
├─ README.zh-CN.md            # 用户说明（中文版）
├─ index.html                 # 入口 HTML（title = Port Process View）
├─ package.json / pnpm-lock.yaml
├─ vite.config.ts             # 固定端口 1420；忽略 watch src-tauri
├─ uno.config.ts              # UnoCSS preset-wind3 + shortcuts
├─ tsconfig.json / tsconfig.node.json
├─ src/                       # Vue 3 前端
│  ├─ main.ts                 # 应用入口
│  ├─ App.vue                 # 布局骨架 + naive-ui 亮/暗主题覆盖（LearnHub 风格）
│  ├─ style.css               # 设计 token（--pv-*）+ 贴纸组件覆盖 + 字体
│  ├─ types.ts                # ConnectionInfo / ProcessInfo（Rust 镜像类型，camelCase）
│  ├─ components/
│  │  ├─ TitleBar.vue         # 无边框窗口：可拖拽区 + 交通灯（最小化/最大化/关闭）
│  │  ├─ HeroHeader.vue       # 品牌区 + 4 个统计块 + 设置入口
│  │  ├─ FilterBar.vue        # 协议/状态下拉 + 搜索框 + 刷新按钮
│  │  ├─ ProcessList.vue      # 进程表格：排序/展开连接明细/结束进程
│  │  └─ SettingsModal.vue    # 设置弹窗：通用 / 赞助 / 关于；赞助含二维码、「一毛也是爱」、关注我们、代码仓库；关于含标识/版本/检查更新/技术栈
│  ├─ composables/
│  │  ├─ useTheme.ts          # 主题单例（system/light/dark），同步 <html data-theme>
│  │  └─ useUpdate.ts         # 升级检查单例（Gitee Releases API）
│  ├─ i18n/index.ts           # zh/en 字典 + t()，localStorage 键 port-view-lang
│  ├─ stores/connections.ts   # Pinia store：数据获取/搜索/筛选/统计/结束进程
│  └─ utils/open.ts           # openExternal（Tauri opener / 浏览器回退）
├─ src-tauri/                 # Rust 后端
│  ├─ src/
│  │  ├─ main.rs              # 仅调用 port_view_lib::run()
│  │  ├─ lib.rs               # 5 个 Tauri 命令 + 系统托盘 + CPU 基线预热 + Win11 圆角
│  │  └─ netinfo.rs           # WinAPI 采集：连接/进程/内存/CPU + 单元测试
│  ├─ build.rs                # 内嵌 requireAdministrator 清单（UAC 提权）
│  ├─ tauri.conf.json         # 窗口 1280x800、无装饰、transparent、identifier
│  ├─ capabilities/default.json  # 权限：core + opener + 窗口控制
│  └─ Cargo.toml              # windows crate 0.62（按需 feature）
├─ public/
│  ├─ fonts/                  # Fredoka / Nunito（本地 woff2，离线可用）
│  └─ qr/                     # 赞助二维码（alipay.png / wechat.png）
├─ dist/                      # 前端构建产物（勿手改）
└─ docs/                      # ADR-001~005、GLOSSARY.md、design-system/MASTER.md
```

## 5. 架构与数据流

### 5.1 IPC 边界（前后端唯一通道）

Rust 侧暴露 5 个 Tauri 命令（`lib.rs`）：

| 命令 | 返回 | 说明 |
|---|---|---|
| `get_connections()` | `ConnectionInfo[]` | 全部 TCP+UDP 端点（含属主进程信息） |
| `get_processes()` | `ProcessInfo[]` | 系统**全部**进程 + 每进程连接统计 + 内存/CPU |
| `kill_process(pid)` | `String` | 执行 `taskkill /F /PID <pid>`，返回结果或错误 |
| `set_language(lang)` | `()` | 记录界面语言并重建托盘菜单（中英同步） |
| `open_in_explorer(path)` | `()` | 在资源管理器中定位并选中程序文件（右键菜单） |

结构体均 `#[serde(rename_all = "camelCase")]`，前端 `src/types.ts` 有对应镜像接口——**改字段时必须两侧同步**。

### 5.2 数据流

```
用户点「刷新」→ FilterBar → store.refresh()
  → Promise.all(invoke get_connections, invoke get_processes)
  → store 为每行预计算 _searchIndex（含拼音）、聚合 _ports/_states
  → 前端本地过滤（搜索+协议+状态）→ ProcessList 按 memory 降序渲染
```

- 数据是**一次性快照**，无后台轮询；只有「刷新」或「结束进程成功」后更新。
- 所有筛选 / 统计 / 排序都在前端完成，不额外 IPC。

### 5.3 CPU 计算（两次采样增量法）

- `netinfo.rs` 用进程内全局 `static CPU_LAST: Mutex<Option<CpuSnapshot>>` 存上一次采样的墙钟时刻与各 PID 累计 CPU 时间（内核+用户，100ns）。
- `CPU% = (本次累计 − 上次累计) / (墙钟间隔 × 逻辑核数) × 100`，`clamp(0,100)`、保留两位小数，即任务管理器口径。
- **首屏预热**：`lib.rs::run()` 最早阶段 spawn 一个后台线程先采一次样、丢弃结果，仅为建立基线，使首次打开即有 CPU 值（前端首屏加载自然构成第二次采样）。
- 注意：首次采样（无基线）时该轮 CPU 为 0；两次采样间隔过短（<1ms）也记 0，均属预期。

### 5.4 前端单例模式

`useTheme` / `useI18n` / `useUpdate` 都是**模块级单例**（模块内 `ref` 定义 + `useXxx()` 返回），保证跨组件共享同一状态并自动响应式。新增共享 UI 状态请沿用此模式，不要在组件内各自 `ref`。

## 6. 关键约定

### 6.1 新增界面文案必须走 i18n

- 所有用户可见文案在 `src/i18n/index.ts` 的 `zh` / `en` **两个字典**里各加一条 key，用 `t("key", {var})` 读取；带变量用 `{var}` 占位。
- TCP 状态枚举（`LISTENING` 等）保持英文不译。
- 相关 localStorage 键：语言 `port-view-lang`、主题 `port-view-theme`、列宽 `port-view-col-widths`（默认值分别为 `zh` / `system` / 组件内默认列宽）。
- 切换语言（`setLang`）会 `invoke("set_language")` 通知 Rust 重建**系统托盘菜单**文案（文案定义在 `lib.rs` 的 `tray_labels`，按 `LANG` 静态变量取 zh/en）。

### 6.2 视觉必须贴合设计系统

- 设计 token 一律用 CSS 变量 `--pv-*`（定义在 `src/style.css`，暗色在 `[data-theme="dark"]` 下覆盖）。
- 贴纸风格：卡片/按钮/输入框统一 3px 深色粗边框（`--pv-stroke`）+ 偏移硬阴影（`--pv-sticker-shadow`）；pill 标签用 `.pv-pill`，按钮用 `.pv-btn`。
- 组件级 naive-ui 覆盖有两处：`App.vue`（主题色/圆角）与 `style.css`（`!important` 贴纸边框）。改样式时**不要破坏贴纸风格**；标题/数字用 `.font-heading`（Fredoka）。
- 全局基准见 `docs/design-system/MASTER.md`，含反模式清单（不用 emoji 作图标、浅色正文对比度 ≥ 4.5:1 等）。

### 6.3 数据模型一致性

- Rust `netinfo.rs` 中 `ConnectionInfo` / `ProcessInfo`（camelCase 序列化）↔ 前端 `src/types.ts` 一一对应；store 行还会附加内部字段 `_searchIndex` / `_ports` / `_states`，这些是 UI 用内部字段，不要塞进 IPC 契约。
- 「进程 TCP/UDP 计数之和 = 连接总数」由 `cargo test` 的 `test_get_processes` 断言守护，改采集逻辑别破坏该不变量。

### 6.4 浏览器开发环境兜底

`pnpm dev` 在纯浏览器运行时没有 Tauri IPC：`TitleBar` 的窗口操作、`open.ts`、`useUpdate` 的版本获取等都必须有 try/catch 或兜底，**不要引入运行时必然报错的代码**。

## 7. 测试

- **Rust 单元测试**：`src-tauri/src/netinfo.rs` 内 `#[cfg(test)]`，含 `test_get_connections` 与 `test_get_processes`。在 `src-tauri/` 下 `cargo test` 运行（会真实调用 WinAPI，建议管理员环境）。
  - 守护项：至少采集到连接、含 System(pid 4) 与当前测试进程、连接统计与连接总数一致、自身内存 >1MB、CPU 值落在 0–100、二次采样能算出非零 CPU。
- **前端**：无独立测试框架；以 `pnpm build`（`vue-tsc --noEmit` 类型检查）+ 手动刷新验证为主。

## 8. 常见任务修改入口（速查）

| 想做什么 | 改哪里 |
|---|---|
| 加一列展示新字段 | ① Rust `ProcessInfo`/`ConnectionInfo` + ② `types.ts` + ③ `ProcessList.vue` 的 `columns`（含 `COL_DEFAULTS` 默认宽/最小宽）与模板 + ④ i18n 字典 |
| 改默认排序字段/方向 | `ProcessList.vue` 的 `sortKey`/`sortDir`（当前默认 `"cpu"` 降序） |
| 加/改右键菜单项 | `ProcessList.vue` 的 `openCtx` / `ctxOpenLocation` / `ctxKill` + i18n 字典 |
| 改托盘菜单文案/行为 | `lib.rs`：`tray_labels` / `rebuild_tray` / `show_main_window`（菜单「显示」）/ `toggle_main_window`（左键显隐）；关闭到托盘在 `TitleBar.vue` 的 `onCloseRequested` |
| 新增界面文案 | `src/i18n/index.ts`（zh + en 两处） |
| 新增 Tauri 命令 | `lib.rs` 定义 `#[tauri::command]` + 注册进 `generate_handler!` + 前端 `invoke` |
| 改窗口行为（尺寸/无边框/透明） | `src-tauri/tauri.conf.json`（窗口尺寸、`decorations`、`transparent`）；权限加在 `capabilities/default.json` |
| 加设置项 | `SettingsModal.vue` 对应 Tab + `useTheme`/`useI18n`/`useUpdate` 或新单例 |
| 改搜索/筛选逻辑 | `src/stores/connections.ts`（`buildSearchIndex` / `filteredProcesses`） |
| 改采集算法 | `src-tauri/src/netinfo.rs`（注意保持连接统计不变量与 CPU 基线） |
| 换版本号 | `package.json` + `Cargo.toml` + `tauri.conf.json` 三处同步（升级检查读的是 Tauri 运行时版本） |
| 改升级检查 | `src/composables/useUpdate.ts`（Gitee Releases API，404 视为已是最新） |

## 9. 注意事项与陷阱

1. **管理员权限是运行前提**：`build.rs` 内嵌 `requireAdministrator`；`kill_process` 与读取受保护进程路径/版本信息依赖它。不要去掉该清单。
2. **仅 IPv4**：采集只走 `AF_INET`，IPv6 连接不在当前范围（ADR-002 已说明）。
3. **`windows` crate 版本错位**：`lib.rs::apply_window_round` 中，tauri 内部用 windows 0.61、本项目锁 0.62，需用裸指针重建本版本 `HWND`——改该函数时注意版本差异。
4. **生成目录勿手改**：`dist/`（前端产物）、`src-tauri/gen/schemas/`（schema 自动生成）、`Cargo.lock`/`pnpm-lock.yaml`（锁文件）。
5. **`FilterBar.vue` 的「其他」标签**用 `t("filter.allStatus") === "全部状态" ? "其他" : "Other"` 判断语言——属于既有脆逻辑，改 i18n 时留意别破坏。
6. **CPU 首屏依赖预热线程**：删除 `lib.rs` 里的预热 `spawn` 会导致首屏 CPU 全为 0，不要误删。
7. **系统关键进程警示**：`ProcessList.vue::isCritical` 列出 System/svchost/csrss/wininit/services/lsass 等，结束进程的确认文案由此触发——新增高危进程类别时在这里补充。
8. **统计口径**（`GLOSSARY.md`）：TCP 总数含 LISTENING；监听端口 = LISTENING 条目数；进程 = 全部进程数。改统计前先核对口径。
9. 未添加 LICENSE，著作权归作者所有，涉及开源/分发需先与作者确认。
10. **关闭窗口 = 隐藏到托盘**：`TitleBar.vue` 拦截 `onCloseRequested` 改为 `hide()`；真正退出用托盘"退出"（Rust `app.exit(0)`）。新增窗口关闭逻辑时别破坏该行为。
11. **托盘语言同步**：托盘菜单文案在 Rust 侧（`LANG` 静态 + `tray_labels`），前端 `setLang` 会 `invoke("set_language")` 重建菜单；新增托盘文案要同时维护中英两套。
12. **列宽持久化**：`ProcessList.vue` 列宽存 `localStorage["port-view-col-widths"]`，损坏/越界数据自动回退默认值。
13. **README 已拆分为两份**：`README.md`（英文）与 `README.zh-CN.md`（中文），改 README 时两边都要更新。
14. **`open_in_explorer` 必须用 `raw_arg`**：explorer 定位文件依赖 `/select,"路径"` 精确语法（含引号）。用普通 `.arg()` 传参会被 Rust 二次转义，表现为"只打开文件管理器、不选中文件"。改该函数时不要去掉 `std::os::windows::process::CommandExt::raw_arg`。
