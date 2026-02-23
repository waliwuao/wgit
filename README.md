# 🚀 wgit (Wally's Git Assistant)

`wgit` 是一个为开发者设计的 Git 工作流辅助工具。它通过 **交互式 TUI 界面** 引导你完成从初始化到发布的所有步骤，确保你的 Git 记录清晰、规范且符合 Git Flow 标准。

![Language](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)

---

## ✨ 核心特性

- 🎨 **专业视觉体验**: 精心设计的终端排版，命令输出主次分明，描述信息自动弱化。
- 📝 **结构化提交**: 交互式表单引导填写 `Scope`、`Subject` 和 `Body`，再也不用担心提交信息乱糟糟。
- 🌿 **严谨的工作流**: 
    - 自动保护 `main` 和 `develop` 分支，防止误操作。
    - 规范 `feature/`, `hotfix/`, `release/` 的生命周期。
- 🔄 **智能同步**: 一个命令自动处理 Pull 和 Push。
- ⏪ **可视化撤销**: 直观的 `log` 列表，支持 `Soft`、`Mixed`、`Hard` 三种回退模式。

---

## 📦 安装指南

你可以直接从 [Releases](https://github.com/waliwuao/wgit/releases/tag/v0.1.0) 下载预编译的二进制文件，无需安装 Rust 环境。

### 🐧 Linux (Ubuntu, CentOS, Arch 等)

在终端中执行以下命令进行安装：

```bash
# 1. 下载二进制文件
curl -L https://github.com/waliwuao/wgit/releases/download/v0.1.0/wgit-linux-amd64 -o wgit

# 2. 赋予执行权限
chmod +x wgit

# 3. 移动到系统路径 (以便全局调用)
sudo mv wgit /usr/local/bin/

# 4. 验证安装
wgit --version
```

### 🪟 Windows (PowerShell)

建议通过 PowerShell 进行安装并添加到环境变量：

```powershell
# 1. 下载文件
Invoke-WebRequest -Uri "https://github.com/waliwuao/wgit/releases/download/v0.1.0/wgit-windows-amd64.exe" -OutFile "wgit.exe"

# 2. 将 wgit.exe 移动到一个固定的文件夹 (例如 C:\tools)
New-Item -ItemType Directory -Force -Path "C:\tools"
Move-Item -Path "wgit.exe" -Destination "C:\tools\wgit.exe"

# 3. 将该路径添加到环境变量 (永久生效)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\tools", "User")

# 注意：设置环境变量后，请重启 PowerShell 窗口
wgit --version
```

---

## 🎮 使用方法

直接输入 `wgit` 即可进入 **交互式主菜单**，通过上下键选择功能。也可以直接使用子命令：

| 命令 | 功能描述 (按字数排序) |
| :--- | :--- |
| `wgit add` | 交互式选择并暂存修改过的文件 |
| `wgit init` | 初始化 Git Flow 环境及分支保护钩子 |
| `wgit sync` | 智能同步当前分支（Pull & Push） |
| `wgit undo` | 基于记录回退到指定的提交点 |
| `wgit exit` | 退出 wgit 辅助助手 |
| `wgit config` | 管理远程仓库地址及工作流偏好设置 |
| `wgit branch` | 管理功能分支（Start/Finish/Switch） |
| `wgit commit` | 交互式生成符合规范的结构化提交信息 |

---

## 💡 典型场景

### 1. 开启新功能开发
1.  运行 `wgit branch`。
2.  选择 `Start`。
3.  选择 `feature` 类型并输入功能名（如 `user-auth`）。
4.  `wgit` 会自动创建 `feature/user-auth` 并为你完成切换。

### 2. 规范化提交代码
1.  运行 `wgit add` 勾选你想要提交的文件。
2.  运行 `wgit commit`。
3.  在 **STRUCTURED COMMIT EDITOR** 中填写信息。
4.  移动光标到 `[ CONFIRM AND COMMIT ]` 按回车。

### 3. 完成并合并分支
1.  在功能分支下运行 `wgit branch`。
2.  选择 `Finish`。
3.  `wgit` 会自动将代码合并到 `develop` 分支（如果是 release 分支则合并到 `main` 并引导打 Tag），随后删除该功能分支。

---

## 🛡 分支保护机制

为了防止代码库混乱，`wgit init` 会在 `.git/hooks/pre-commit` 安装钩子：
- 🚫 **禁止** 直接在 `main`/`master` 分支提交代码。
- 🚫 **禁止** 直接在 `develop` 分支提交代码。
- ✅ 强制开发者使用功能分支，通过 `wgit branch finish` 进行规范合并。

---

## 🤝 贡献与反馈

如果你有任何建议或发现了 Bug，欢迎提交 [Issue](https://github.com/waliwuao/wgit/issues)。

**作者**: [Waliwuao](https://github.com/waliwuao)  
**许可证**: [MIT](LICENSE)