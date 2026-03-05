# wgit 最小测试矩阵与 CI 门禁建议

目标是以最小成本建立“可回归、可发布”的质量保障，优先覆盖初学者最易受影响的关键路径。

## 测试矩阵（最小可用）

| 层级 | 覆盖对象 | 建议数量 | 价值 |
|---|---|---:|---|
| 单元测试 | 纯函数与规则校验（分支名、tag、版本比较、消息格式） | 20-30 | 快速回归，定位精确 |
| 组件测试 | `git` 输出解析与配置加载回退逻辑 | 8-12 | 降低边界输入风险 |
| 流程测试 | `start/commit/sync/undo/finish` 关键状态流（mock git） | 6-10 | 保证引导流程稳定 |
| 冒烟测试 | CLI 子命令可调用与 `--help` 输出稳定性 | 1-2 | 防止发布包不可用 |

## 高优先级用例清单

1. `start` 分支命名非法输入与重复分支处理。
2. `commit` 在空 staged、空 subject 下的行为。
3. `finish` 冲突后 abort/continue 分支路径。
4. `sync` 在无 upstream、有 stash、rebase 冲突时的提示闭环。
5. `undo hard` 的确认与目标选择逻辑。
6. 配置缺失、配置空值、配置非法格式时的回退策略。

## CI 门禁（建议分三阶段）

## 阶段 1：基础门禁（立即可做）

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- 触发条件：`push` + `pull_request`

通过标准：三项全绿才可合并。

## 阶段 2：稳定性门禁（第二阶段）

- Linux + macOS 双平台测试（Windows 可后置）。
- 增加最小集成测试（临时仓库 + mock git 命令）。
- 对关键命令引入快照测试（提示文案和交互分支）。

## 阶段 3：发布门禁（与 release 联动）

- 发布前必须通过主分支 CI。
- 对发布二进制增加完整性校验（checksum）并产出校验文件。
- 为 `update` 机制增加“下载校验失败”的自动阻断和提示。

## 推荐工作流拆分

- `ci.yml`：质量门禁（lint/test）。
- `release.yml`：发布构建（仅在 tag 触发）。
- 可选 `nightly.yml`：慢测试与兼容性巡检。

## 失败时提示规则

CI 输出应优先给新手可执行信息：

1. 哪一步失败（fmt/clippy/test）。
2. 本地复现命令。
3. 一句话修复方向。

示例：

- `Step failed: clippy`
- `Run locally: cargo clippy -- -D warnings`
- `Hint: remove unused code or handle Result explicitly`
