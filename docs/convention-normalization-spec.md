# wgit 规范策略统一方案

本文用于统一 `wgit` 在分支、提交、标签与配置语义上的规则，避免“配置一套、代码一套、提示文案一套”。

## 设计目标

- 规则唯一来源：同一规范只在一个地方定义。
- 行为可预测：默认值、校验、文案一致。
- 新手可理解：失败提示包含原因和修复建议。

## 1. 分支规范统一

## 1.1 默认分支语义

- 引入统一概念：`primary_branch`（默认 `main`）。
- 兼容 `master` 仅作为迁移能力，不作为长期默认文案。
- `init` 行为：
  - 若存在 `master` 但不存在 `main`，提示并确认是否重命名为 `main`。
  - 若仓库尚无分支，设置 `HEAD -> primary_branch`。

## 1.2 保护分支配置

建议将 `.git/wgit.toml` 规范化为：

```toml
[flow]
primary_branch = "main"

[safety]
protected_branches = ["main"]
allow_commit_on_protected = false
```

规则要求：

- `protected_branches` 默认包含 `primary_branch`。
- 若配置为空，回退到默认并给出提示。
- 所有命令共用同一 `is_protected_branch` 逻辑。

## 1.3 分支命名规则

### 建议格式

- `<type>/<slug>`
- `type` 默认允许：`feature|bugfix|hotfix|release`
- `slug` 仅允许：`[a-z0-9._-]`，且不能以 `/`、`.` 结尾，不能包含 `..`。

### 校验策略

- 在 `start` 输入后先做本地正则校验，再执行 `git check-ref-format --branch` 终检。
- 报错文案统一为：
  - 输入值
  - 不通过原因
  - 合法示例（如 `feature/login-form`）

## 2. 提交规范统一

## 2.1 提交类型

固定集合（默认）：`feat|fix|docs|refactor|test|chore`

- 类型列表从配置读取，可扩展但不允许空集合。
- `commit` 与 `finish`（merge message）共用编辑器结构和 header 生成逻辑。

## 2.2 提交消息模板

统一模板：

- Header: `type(scope): subject` 或 `type: subject`
- Body: 可选，多行描述变更动机与影响

统一校验：

- `subject` 必填，去首尾空格后长度 `1..72`。
- Header 禁止换行。
- 若不满足，给出结构化错误提示和示例。

## 3. Tag 规范统一

- 默认采用 `vMAJOR.MINOR.PATCH`（例如 `v1.2.3`）。
- 允许通过配置切换是否强制 `v` 前缀。
- 在 `finish` 打 tag 时：
  - 展示最新 tag
  - 校验新 tag 递增
  - 失败时说明“当前版本”和“候选版本”

## 4. 文案与错误格式统一

统一输出结构：

1. `Action`: 当前正在执行什么。
2. `Reason`: 为什么失败/为何提示。
3. `Next`: 推荐下一步（最多 3 条）。

示例：

- `Action: create branch feature/login-form`
- `Reason: branch name fails git check-ref-format`
- `Next: use lowercase letters, digits, -, _, .`

## 5. 实施顺序（建议）

1. 新增配置字段与默认值兼容层（保持旧配置可读）。
2. 抽取统一校验模块（branch/commit/tag）。
3. 将 `start/commit/finish/init` 迁移到统一规则入口。
4. 最后统一提示文案与帮助信息。
