# 🚀 wgit (Wally's Git Assistant)

`wgit` 是一个为开发者设计的 Git 工作流辅助工具。它通过 **交互式 TUI 界面** 引导你完成从初始化到发布的所有步骤，确保你的 Git 记录清晰、规范且符合 Git Flow 标准。

![Language](https://img.shields.io/badge/language-Rust-orange)
![Release](https://img.shields.io/badge/version-0.1.1-green)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)

---

## ✨ 核心特性

- 🎨 **专业视觉体验**: 精心设计的终端排版，命令输出主次分明，描述信息自动弱化。
- 📝 **结构化提交**: 交互式表单引导填写 `Scope`、`Subject` 和 `Body`。
- 🌿 **严谨的工作流**: 自动保护 `main` 和 `develop` 分支，规范功能分支生命周期。
- 🔄 **一键同步**: 一个命令自动处理 Pull 和 Push，支持多远程仓库选择。
- ⏪ **可视化撤销**: 直观的 `log` 列表，支持 `Soft`、`Mixed`、`Hard` 三种模式。

---

## 📦 安装指南

你可以直接从 [Releases](https://github.com/waliwuao/wgit/releases) 下载预编译的静态二进制文件，无需安装 Rust 环境。

### 🐧 Linux (Ubuntu, CentOS, Arch, Debian 等)

```bash
# 下载最新的 v0.1.1 静态二进制文件
curl -L https://github.com/waliwuao/wgit/releases/download/v0.1.1/wgit-linux-amd64 -o wgit_tmp

# 赋予权限并移动到系统路径
chmod +x wgit_tmp
sudo mv wgit_tmp /usr/local/bin/wgit

# 刷新 shell 路径缓存
hash -r

# 验证安装
wgit --version
```

### 🪟 Windows (PowerShell)

```powershell
# 1. 下载文件
Invoke-WebRequest -Uri "https://github.com/waliwuao/wgit/releases/download/v0.1.1/wgit-windows-amd64.exe" -OutFile "wgit.exe"

# 2. 移动到固定文件夹并添加到环境变量 (例如 C:\tools)
New-Item -ItemType Directory -Force -Path "C:\tools"
Move-Item -Path "wgit.exe" -Destination "C:\tools\wgit.exe"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\tools", "User")

# 注意：设置完成后请重启 PowerShell 窗口
wgit --version
```

---

## 🎮 快速开始

直接输入 `wgit` 即可进入 **交互式主菜单**。

| 命令 | 功能描述 |
| :--- | :--- |
| `wgit add` | 交互式选择并暂存修改过的文件 |
| `wgit init` | 初始化 Git Flow 环境及分支保护钩子 |
| `wgit sync` | 智能同步当前分支（Pull & Push） |
| `wgit undo` | 可视化回退到指定的提交点 |
| `wgit branch` | 管理功能分支（Start/Finish/Switch） |
| `wgit commit` | 交互式生成符合规范的结构化提交信息 |

---

## 🛡 分支保护机制

`wgit init` 会在 `.git/hooks/pre-commit` 安装强制钩子：
- 🚫 **禁止** 在 `main` / `develop` 分支直接提交代码。
- ✅ 强制使用功能分支开发，通过 `wgit branch finish` 自动合并，保持主干整洁。

---

**作者**: [Waliwuao](https://github.com/waliwuao)  
**许可证**: [MIT](LICENSE)