# cx — AI 原生代码管理工具

`cx` 是一个工作在 git 之上的 CLI 工具，为 AI 辅助编码提供结构化上下文管理。
它将 AI 会话视为一等公民，支持 snapshot 追踪、按意图分组和审查——同时完全兼容
你的 git 工作流。

## 快速开始

```bash
# 在当前 git 仓库初始化
cx init

# 开始一个 AI 会话
cx start

# 编码并记录变更
cx apply -m "添加支付宝签名逻辑" --intent feature,scope=payment
cx apply -m "抽取签名工具函数" --intent refactor,scope=crypto,risk=medium

# 查看状态并审查
cx status
cx log
cx end --merge
```

## 命令参考

| 命令 | 说明 |
|------|------|
| `init` | 在当前 git 仓库初始化 .cx/ 元数据存储 |
| `start` | 开始一个新的 AI 会话，自动创建分支 |
| `apply -m <msg> --intent <spec>` | 将暂存的变更记录为带语义标签的快照 |
| `status` | 显示当前会话概览，按意图分组 |
| `end --merge / --abandon` | 结束会话：合入主分支或丢弃 |
| `log [session-id]` | 查看快照/会话历史 |
| `review [snapshot-id]` | 按语义意图分组审查变更 |

## Intent 参数格式

```
--intent <类型>,scope=<作用域>[,risk=<风险>]
```

- **类型**（必填）: `feature` | `fix` | `refactor` | `style` | `docs` | `dependency` | `test` | `chore`
- **作用域**（可选）: 任意标签如 `payment`、`login`
- **风险**（可选，默认 `low`）: `low` | `medium` | `high`

## Codex 技能

`cx` 自带一个 Codex 技能，让 [Codex](https://codex.ai) 在编码过程中自动使用 `cx`。

### 位置

```
agents/skills/cx-workflow/
├── SKILL.md               # Codex 读取的技能指令
└── agents/openai.yaml     # UI 元数据
```

### 工作原理

当 Codex 在已初始化 `.cx/` 的仓库中编码时，技能会指示 Codex 自动执行以下流程：

1. **开始会话**: `cx start "<prompt>"` — 创建 session 分支
2. **记录变更**: `git add <文件>` + `cx apply -m "<说明>" --intent <类型>,scope=<作用域>` — 每次变更加语义标签
3. **结束会话**: `cx end --merge` — 合入主分支

### 安装

将技能链接到 Codex 发现目录即可启用：

```bash
ln -s /绝对路径/cx/agents/skills/cx-workflow ~/.codex/skills/cx-workflow
```

链接后，Codex 在带 `.cx/` 的仓库中编码时会自动加载并使用 `cx`。

## 架构

```
cx (CLI 二进制)
├── cli.rs          — clap 子命令定义
├── session.rs      — 核心会话逻辑（start / apply / end / status）
├── snapshot.rs     — 快照创建与内容哈希
├── context.rs      — 数据模型（Snapshot, Context, IntentKind 等）
├── store.rs        — .cx/ 存储层（基于 JSON 的内容寻址）
├── git_api.rs      — Git 命令封装
└── display.rs      — 格式化输出（status, log, review）
```

### 存储结构

```
.cx/sessions/<session_id>/
├── meta.json           # 会话元数据（状态、基础分支）
└── snapshots/
    └── <序号>.json     # 快照（文件名 = commit 序号）
```

### Git 集成

`.cx/` 被 git 跟踪管理。每次 `cx apply` 把代码和 `.cx/` 元数据放在同一个 commit 里。
会话分支就是真正的 git 分支。

```
commit bb7e0aa (cx/s_xxxxxx)
├── pay.rs                              # 代码变更
└── .cx/sessions/s_xxxxxx/              # 会话元数据
    ├── meta.json
    └── snapshots/1.json
```

- **commit → snapshot**: `git show --name-only <hash>` 输出的文件路径中
  包含了 session ID 和 turn 编号
- **snapshot → 当前状态**: 当前分支名 `cx/<sid>` 直接标识活跃会话
- **merge**: `cx end --merge` 后，所有元数据都在主分支上

### 设计决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 语言 | Rust | 单二进制，零运行时依赖 |
| Git 交互 | CLI 子进程 | 避免 libgit2 编译复杂度 |
| 存储 | JSON + 序号文件 | 零依赖，git history 可追溯 |
| 会话标识 | Git 分支名 | 无需全局状态文件 |
| .cx/ 管理 | 由 git 跟踪 | clone 后元数据不丢失 |

## 构建

```bash
cargo install --path .
```

## 许可

MIT
