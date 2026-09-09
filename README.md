# PortView

[English](#-english) · [中文](./README.zh-CN.md)

> The full Chinese version lives in **[README.zh-CN.md](./README.zh-CN.md)**.

---

# 🇬🇧 English

A lightweight **Windows** desktop tool that lists every TCP/UDP endpoint, the process that owns each port, and — for every process — its memory footprint and CPU usage. Kill a process, fuzzy-search by name / pinyin / PID / port, and sort by resource usage, all in one sticker-style interface.

Built with **Tauri 2 + Vue 3 + TypeScript + Vite + UnoCSS + Pinia + naive-ui**.

> Repository: <https://gitee.com/ShiXiongZhiDao/port-view>

## ✨ Features

- **Process ↔ port unified view** — every running process is listed (including processes with no network connection, e.g. Everything), together with its TCP count, UDP count, listening ports and an aggregated port summary. Expand a row to inspect every raw connection (protocol / local address / remote address / state).
- **Per-process memory & CPU** — physical working set and CPU percentage (same convention as Task Manager) for each process. Click the **CPU / Memory** column header to sort; results are sorted by **CPU descending by default**.
- **Resizable columns** — drag any column header edge to adjust its width; widths are remembered between launches.
- **Right-click context menu** — on any process row: **Open file location** (reveals the program in Explorer) and **Kill process** (with confirmation).
- **Fuzzy search** — match by process name, software name, full path, PID, port, and Chinese **pinyin / initials** (powered by `pinyin-pro`).
- **Filters** — filter by protocol (All / TCP / UDP) and by connection state (LISTENING / ESTABLISHED / TIME_WAIT / CLOSE_WAIT / Other), stacked with the search query.
- **Manual refresh** — data is a snapshot taken only when you press **Refresh** (or after killing a process); there is no background polling.
- **Terminate process** — `taskkill /F` with a confirmation popover; critical system processes get an extra red warning.
- **Bilingual UI** — switch between 简体中文 and English in Settings; the choice is persisted (the tray menu follows it too).
- **Light / Dark / System theme**, persisted.
- **Check for updates** — queries the Gitee Releases API, compares semantic versions, and links to the new release.


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

The NSIS installer is emitted under `target/release/bundle/nsis/` as **`PortView_<version>_x64-setup.exe`**. After installation the program — executable, install folder, Start Menu shortcut, taskbar/tray icon and uninstaller — is named **PortView** and uses the custom green "PV" app icon. The installer wizard, the uninstaller and the app itself never fall back to the default Tauri icon. To regenerate the whole icon set from a new 1024×1024 source, run `pnpm tauri icon <source.png>` (the source must be a real PNG).

## 🖱️ Usage

1. Press **Refresh** to take a fresh snapshot of processes and connections.
2. Type in the search box to filter by name / pinyin / PID / port; use the two dropdowns to narrow by protocol and state.
3. Click the **CPU** or **Memory** header to change sort field and direction (desc ↔ asc); the list is sorted by **CPU descending** by default. Drag any column header edge to resize the column.
4. Right-click a process row to **open its file location** or **kill** it; click **Connections** on a row to expand its raw endpoints, or click **Kill** to terminate it after confirmation.
5. Open **Settings** via the gear icon to change theme/language, open the repository, check for updates, or view version info.


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
│  │  ├─ lib.rs              # Tauri commands, tray, window rounded corners
│  │  └─ netinfo.rs          # WinAPI collection (connections, processes, memory/CPU)
│  ├─ capabilities/          # Tauri permissions
│  └─ tauri.conf.json
├─ public/
│  ├─ fonts/                 # Fredoka / Nunito (self-hosted)
```

## 📄 License

No license file has been added yet. All rights reserved by the author until one is provided.
