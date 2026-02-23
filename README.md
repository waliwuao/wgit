# 🚀 wgit (Wally's Git Assistant)

`wgit` 是一个为极致开发体验设计的 Git 工作流辅助工具。它不仅提供美观的 **TUI 交互界面**，更在底层实现了 **事务安全**、**自动状态管理** 和 **一键自更新**，确保你的 Git 记录始终保持规范。

![Language](https://img.shields.io/badge/language-Rust-orange)
![Release](https://img.shields.io/badge/version-0.3.0-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)

---

## ✨ 核心特性 (v0.3.0 旗舰版)

- 🔄 **一键自更新 (`wgit update`)**: **核心更新！** 彻底告别手动下载。只需一个命令，`wgit` 会自动从 GitHub 获取最新版并完成原地无缝替换。
- 🛡️ **事务安全检查**: 执行 `branch finish` 或 `sync` 前自动检测工作区，杜绝因本地冲突导致的合并中断。
- ⚡ **智能自动暂存 (Auto-Stash)**: `sync` 时自动执行 `stash push/pop`。即使你有未提交的临时改动，也能顺滑执行同步。
- 🔍 **全局模糊搜索**: 所有分支列表、文件列表均支持实时模糊匹配，搜索体验快如闪电。
- 📦 **语义化版本建议**: 发布时自动获取最新 Tag 并推导 `Patch/Minor/Major` 建议值，再也不怕打错版本号。
- 🧱 **非侵入式 Hook**: 使用标记块管理 `.git/hooks`，在强化保护的同时，绝不破坏你原有的 Husky 或自定义配置。
- 🔒 **本地配置隔离**: 偏好设置存储在 `.git/wgit.json`。不污染项目根目录，本地私有配置不会进入 Git 历史。

---

## 📦 安装与升级

### 🆙 首次安装
如果你是第一次使用 `wgit`，请从 [Releases](https://github.com/waliwuao/wgit/releases) 下载对应平台的二进制文件。

#### 🐧 Linux
```bash
curl -L https://github.com/waliwuao/wgit/releases/download/v0.3.0/wgit-linux-amd64 -o wgit
chmod +x wgit
sudo mv wgit /usr/local/bin/
```

#### 🪟 Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri "https://github.com/waliwuao/wgit/releases/download/v0.3.0/wgit-windows-amd64.exe" -OutFile "wgit.exe"
```

### 🚀 后续升级
**安装 v0.3.0 之后，你只需要运行：**
```bash
wgit update
```

---

## 🎮 命令全览

| 命令 | 特性描述 |
| :--- | :--- |
| **`wgit update`** | **自进化**: 自动检测、下载并替换至最新版本。 |
| `wgit sync` | **智能同步**: 自动 Stash 流程，一键 Pull & Push。 |
| `wgit branch` | **工作流**: 分支切换支持模糊搜索，Release 支持版本建议。 |
| `wgit commit` | **规范化**: 交互式引导生成符合规范的结构化提交。 |
| `wgit add` | **增强选择**: 模糊过滤文件列表，安全处理复杂文件名。 |
| `wgit undo` | **回退指南**: 可视化回退 `log` 或 `reflog`。 |

---

## 🛡 工作流与保护

`wgit` 会自动在当前仓库安装 `pre-commit` 钩子：
- 🚫 **严禁** 在 `main` / `develop` 分支直接提交。
- ✅ **推荐**: 使用 `branch start` 创建功能分支，开发完成后通过 `branch finish` 自动合并。

---

**作者**: [Waliwuao](https://github.com/waliwuao) 
 