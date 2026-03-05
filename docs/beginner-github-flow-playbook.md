# wgit 初学者 GitHub Flow 操作手册（骨架）

本文面向 Git 初学者，目标是用最少记忆成本完成一次标准协作闭环。

## 你将完成什么

1. 初始化并确认默认分支。
2. 创建功能分支并提交代码。
3. 同步远程并发起 PR（建议新增 `wgit pr` 后接入）。
4. 处理冲突与评审。
5. 合并后回收分支并同步本地。

## 场景 A：首次接入仓库

```bash
wgit init
```

预期结果：

- 仓库已初始化（或已检测到 Git 仓库）。
- 默认分支为 `main`（若历史是 `master`，需提示迁移）。
- `.git/wgit.toml` 存在并包含基础安全策略。

## 场景 B：开始需求开发

```bash
wgit start
```

交互建议：

- 先选分支类型：`feature/bugfix/hotfix/release`
- 再填分支名称：建议短横线英文语义名，如 `login-form`

结果示例：`feature/login-form`

## 场景 C：提交改动

```bash
wgit add
wgit commit
```

提交建议：

- 每次提交只做一件事。
- 使用结构化信息：`type(scope): subject`
- 常见 `type`: `feat/fix/docs/refactor/test/chore`

## 场景 D：同步远程

```bash
wgit sync
```

预期：

- 若有未提交改动，工具会先提示并处理 stash。
- 执行 pull(rebase) + push。
- 失败时给出“继续解决冲突”或“中止回退”路径。

## 场景 E：发起与合并 PR（GitHub Flow 关键）

建议主路径（当前项目后续应补齐命令化支持）：

1. `wgit sync` 后在 GitHub 创建 PR。
2. 等待 CI 与 Review 通过。
3. 在 GitHub 合并 PR。
4. 本地切回 `main` 并同步。

在工具层建议新增：

- `wgit pr create`：基于当前分支创建 PR。
- `wgit pr status`：查看 CI/Review/Mergeability。
- `wgit pr open`：打开 PR 页面。

## 场景 F：收尾与回收

若 PR 已合并：

1. 同步 `main`。
2. 删除本地已合并分支。
3. 如需版本发布，再进入 tag 流程。

说明：`finish` 更适合“本地整合流”，建议逐步演进为“PR 合并后清理流”。

## 常见问题与恢复

## 1) rebase 冲突

- 先解决冲突文件。
- `git add <files>`
- `git rebase --continue`
- 再执行 `wgit sync`

## 2) 误选了危险回退（undo hard）

- 先停止继续操作。
- 检查 `git reflog` 找回目标位置。
- 通过回退到安全节点进行恢复。

## 3) 提交信息不规范

- 重新运行 `wgit commit` 并按模板填写。
- subject 保持简短且表达“意图”。

## 团队落地建议

- 将本手册放入仓库主页导航（README）。
- 把关键规范（分支、提交、回退）写入 onboarding 文档。
- 为高风险命令统一添加 Safety Check 提示前缀。
