# 🚀 wgit (Wally's Git Assistant)

`wgit` 是一个为极致开发体验设计的 Git 工作流辅助工具。它不仅提供美观的 **TUI 交互界面**，更在底层实现了 **事务安全**、**自动状态管理** 和 **智能化版本建议**，确保你的 Git 记录清晰、规范且符合生产级标准。

![Language](https://img.shields.io/badge/language-Rust-orange)
![Release](https://img.shields.io/badge/version-0.2.0-blue)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)

---

## ✨ 核心特性 (v0.2.0+)

- 🆙 **一键自更新**: 内置 `wgit update` 命令，自动检测 GitHub 最新版本并完成无缝原地替换。
- 🔍 **全局模糊搜索**: 所有文件选择、分支切换列表均支持实时模糊匹配，大项目操作秒级定位。
- 🛡️ **事务安全保障**: 执行合并 (`finish`) 或同步 (`sync`) 前自动检测工作区。杜绝“半完成”破碎状态。
- ⚡ **自动状态保护 (Auto-Stash)**: `sync` 时自动执行 `stash push/pop`。即使有未提交代码也能平滑同步。
- 📦 **语义化版本建议**: 发布分支完成时，自动根据 **SemVer** 规范推导 `Patch/Minor/Major` 建议值。
- 🧱 **非侵入式 Hook**: 使用标记块技术管理 `.git/hooks`，在增强保护的同时**不破坏**原有 Hook 配置。
- 🔒 **配置隔离**: 偏好设置存储在 `.git/wgit.json`。不污染项目根目录，且本地私有配置不会被误提交。

---

## 📦 安装与升级指南

### 🆕 自动升级 (推荐)
如果你已经安装了 v0.2.0+ 版本，直接运行以下命令即可完成自我进化：
```bash
wgit update
```

### 📥 首次安装或手动覆盖升级
如果你需要手动下载并覆盖旧版本，请根据操作系统执行以下命令：

#### 🐧 Linux
```bash
# 下载最新版并覆盖安装至系统路径
curl -L https://github.com/waliwuao/wgit/releases/download/v0.2.0/wgit-linux-amd64 -o wgit_tmp
sudo install -m 755 wgit_tmp /usr/local/bin/wgit
rm wgit_tmp

# 验证版本
wgit --version
```

#### 🍎 macOS
```bash
# 下载最新版并覆盖安装至系统路径
curl -L https://github.com/waliwuao/wgit/releases/download/v0.2.0/wgit-macos-amd64 -o wgit_tmp
sudo install -m 755 wgit_tmp /usr/local/bin/wgit
rm wgit_tmp
```

#### 🪟 Windows (管理员权限 PowerShell)
```powershell
# 下载并覆盖现有的 wgit.exe (请将路径替换为你实际存放 wgit.exe 的目录)
Invoke-WebRequest -Uri "https://github.com/waliwuao/wgit/releases/download/v0.2.0/wgit-windows-amd64.exe" -OutFile "C:\tools\wgit.exe"
```

---

## 🎮 快速开始

直接输入 `wgit` 即可进入 **交互式主菜单**。

| 命令 | v0.2.0 增强特性 |
| :--- | :--- |
| `wgit update` | **自动进化**: 检测、下载并原地替换最新二进制文件。 |
| `wgit add` | **搜索支持**: 模糊过滤文件；安全处理包含空格的特殊路径。 |
| `wgit sync` | **智能同步**: 自动执行 Stash 流程，支持多远程仓库选择。 |
| `wgit branch` | **自动化流**: 支持分支搜索及 **SemVer 标签助手**。 |
| `wgit commit` | **规范引导**: 交互式表单生成符合 Conventional Commits 标准的提交。 |
| `wgit undo` | **可视化回退**: 直观浏览 log/reflog，安全执行 Hard/Soft 重置。 |
| `wgit config` | **本地偏好**: 管理远程仓库、分支名称及 Review 模式。 |

---

## 🛡 分支保护机制

`wgit init` 会在 `.git/hooks/pre-commit` 安装智能保护逻辑：
- 🚫 **禁止** 在 `main` 或 `develop` 分支直接提交代码。
- ✅ **推荐工作流**: 通过 `branch start` 开启特性分支，完成后通过 `branch finish` 自动执行标准化合并。

---

**作者**: [Waliwuao](https://github.com/waliwuao)  
**许可证**: [MIT](LICENSE)