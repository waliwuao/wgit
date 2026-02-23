`wgit` 是一个为极致开发体验设计的 Git 工作流辅助工具。它不仅提供美观的 **TUI 交互界面**，更在底层实现了 **事务安全**、**自动状态管理** 和 **智能化版本建议**，确保你的 Git 记录清晰、规范且符合生产级标准。

![Language](https://img.shields.io/badge/language-Rust-orange)
![Release](https://img.shields.io/badge/version-0.2.0-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)

---

## ✨ 核心特性 (v0.2.0 更新)

- 🔍 **全局模糊搜索**: 所有文件选择、分支切换列表均支持实时模糊匹配。面对上百个分支或文件，只需输入几个字母即可秒级定位。
- 🛡️ **事务安全保障**: 在执行分支合并 (`finish`) 或同步 (`sync`) 前自动检测工作区状态。杜绝因本地改动导致的“半完成”破碎状态，确保操作的原子性。
- ⚡ **自动状态保护 (Auto-Stash)**: 在 `sync` 过程中自动处理 `stash push/pop`。即使你有未提交的临时代码，也能一键平滑完成远程同步。
- 📦 **语义化版本建议**: 在发布分支完成时，系统会自动获取最新 Tag 并根据 **SemVer (语义化版本)** 规范推导 `Patch/Minor/Major` 建议值，防止版本号输入错误。
- 🧱 **非侵入式 Hook 管理**: 使用独创的标记块 (Marker Blocks) 技术管理 `.git/hooks`。在安装分支保护逻辑的同时，**绝不破坏**你原有的 Husky 或自定义 Hook 配置。
- 🔒 **零污染配置隔离**: 所有的偏好设置均加密存储在 `.git/wgit.json`。不污染项目根目录，且本地配置不会被误提交到远程仓库。

---

## 📦 安装指南

你可以直接从 [Releases](https://github.com/waliwuao/wgit/releases) 下载预编译的二进制文件，即开即用。

### 🐧 Linux
```bash
# 下载 v0.2.0 静态二进制文件
curl -L https://github.com/waliwuao/wgit/releases/download/v0.2.0/wgit-linux-amd64 -o wgit
chmod +x wgit
sudo mv wgit /usr/local/bin/
wgit --version
```

### 🪟 Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri "https://github.com/waliwuao/wgit/releases/download/v0.2.0/wgit-windows-amd64.exe" -OutFile "wgit.exe"
# 将 wgit.exe 移动到你的系统 PATH 目录（例如 C:\Windows\System32 或自定义工具目录）
```

---

## 🎮 快速开始

直接输入 `wgit` 即可进入 **交互式主菜单**。

| 命令 | v0.2.0 增强描述 |
| :--- | :--- |
| `wgit add` | **搜索支持**: 模糊过滤文件；**鲁棒解析**: 安全处理包含空格的特殊路径。 |
| `wgit sync` | **智能同步**: 自动执行 Stash 流程，支持多远程仓库选择。 |
| `wgit branch` | **自动化流**: 支持分支搜索、保护检查及 **SemVer 标签助手**。 |
| `wgit commit` | **规范引导**: 交互式表单生成符合 Conventional Commits 标准的提交。 |
| `wgit undo` | **可视化回退**: 直观浏览 log/reflog，安全执行 Hard/Soft 重置。 |
| `wgit init` | **环境初始化**: 自动配置 Git Flow 结构并安装非侵入式保护钩子。 |

---

## 🛡 分支保护机制

`wgit init` 会在 `.git/hooks/pre-commit` 安装智能保护逻辑：
- 🚫 **禁止** 在 `main` 或 `develop` 分支直接提交代码。
- ✅ **推荐工作流**: 通过 `wgit branch start` 开启特性分支，完成后通过 `wgit branch finish` 自动执行标准化合并与清理。

---

**作者**: [Waliwuao](https://github.com/waliwuao)  
**许可证**: [MIT](LICENSE)