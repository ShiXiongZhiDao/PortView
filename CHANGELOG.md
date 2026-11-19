# Changelog

本文件记录对外发布版本。版本号必须与 `package.json`、`src-tauri/Cargo.toml`、
`src-tauri/tauri.conf.json` 一致，由 `pnpm release` 统一推进。

## [0.2.0] - 2026-11-19

### 新增

- 通过 GitHub Actions 构建并发布 NSIS 安装包与便携版 zip。
- 接入带签名的自动升级：静默检查新版本，用户确认后下载并安装。
- 便携版可检查更新，但不会尝试就地替换自身。

### 变更

- 安装包改为每用户安装（不弹 UAC）；应用运行时仍需管理员权限。
- 源码同步扩展为 GitHub 与 Gitee 两个公开库，更新清单以 Gitee 为主源、GitHub 兜底。

### 注意

- 本次是第一个带签名的版本。旧版 `v0.1.0` 的 Release 与当前源码分支不再对应。
