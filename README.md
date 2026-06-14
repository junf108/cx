# cx — AI-Native Code Management

`cx` is a CLI tool that operates on top of git, providing structured context
management for AI-assisted coding. It treats AI sessions as first-class
citizens with snapshot tracking, intent grouping, and review-by-intent — all
without replacing git.

## Installation

### One-liner (macOS / Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/junf108/cx/main/scripts/install.sh | sh
```

This downloads the pre-built binary for your platform from the [latest release](https://github.com/junf108/cx/releases) and installs it to `/usr/local/bin`.

```bash
# Install to a custom location
CX_INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/junf108/cx/main/scripts/install.sh | sh
```

### Pre-built binaries

Download the archive matching your platform from the [releases page](https://github.com/junf108/cx/releases), then:

```bash
tar xzf cx-<target>.tar.gz
sudo mv cx /usr/local/bin/
```

### Via cargo

If you have the Rust toolchain installed:

```bash
cargo install --git https://github.com/junf108/cx
```

## Quick Start

```bash
# Initialize cx in your git repository
cx init

# Start an AI session
cx start

# Make changes and record them
cx apply -m "Add Alipay signature" --intent feature,scope=payment
cx apply -m "Extract signer utility" --intent refactor,scope=crypto,risk=medium

# Review and finish
cx status
cx review
cx end --merge
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize .cx/ metadata store in the current git repo |
| `start` | Start a new AI session, creating a session branch |
| `apply -m <msg> --intent <spec>` | Record staged changes as a snapshot with semantic labels |
| `status` | Show current session overview grouped by intent |
| `end --merge / --abandon` | End the session: merge to base branch or discard |
| `log [session-id]` | View snapshot / session history |
| `review [snapshot-id]` | Review changes grouped by semantic intent |

## Intent Spec

```
--intent <type>,scope=<label>[,risk=<level>]
```

- **Type** (required): `feature` | `fix` | `refactor` | `style` | `docs` | `dependency` | `test` | `chore`
- **Scope** (optional): arbitrary label like `payment`, `login`
- **Risk** (optional, default `low`): `low` | `medium` | `high`

## Codex Skill

`cx` ships with a Codex skill that makes [Codex](https://codex.ai) automatically use `cx` during AI coding sessions.

### Location

```
agents/skills/cx-workflow/
├── SKILL.md               # Skill instructions consumed by Codex
└── agents/openai.yaml     # UI metadata
```

### How It Works

When Codex is coding in a repository that has `.cx/` initialized, the skill instructs Codex to:

1. **Start a session**: `cx start "<prompt>"` — creates a session branch
2. **Record changes**: `git add <files>` + `cx apply -m "<msg>" --intent <type>,scope=<x>` — records each change as a labeled snapshot
3. **Finish the session**: `cx end --merge` — merges the session branch back to the base branch

### Installation

To enable the skill, link it into Codex's skill discovery directory:

```bash
ln -s /absolute/path/to/cx/agents/skills/cx-workflow ~/.codex/skills/cx-workflow
```

Once linked, Codex will automatically load the skill and use `cx` whenever coding in a `.cx/`-initialized repository.

## Architecture

```
cx (CLI binary)
├── cli.rs          — clap subcommands
├── session.rs      — Core session logic (start / apply / end / status)
├── snapshot.rs     — Snapshot creation and content hashing
├── context.rs      — Data models (Snapshot, Context, IntentKind, etc.)
├── store.rs        — .cx/ storage (content-addressed JSON)
├── git_api.rs      — Git command wrappers
└── display.rs      — Formatted output (status, log, review)
```

### Data Flow

```
cli → session → (store + git_api) → display
```

### Storage Layout

```
.cx/sessions/<session_id>/
├── meta.json           # Session metadata (status, base branch)
└── snapshots/
    └── <turn>.json     # Snapshot (filename = commit sequence number)
```

### Git Integration

`.cx/` is tracked by git. Every `cx apply` commits code and `.cx/` metadata
together in a single commit. Session branches are real git branches.

```
commit bb7e0aa (cx/s_xxxxxx)
├── pay.rs                              # Code changes
└── .cx/sessions/s_xxxxxx/              # Session metadata
    ├── meta.json
    └── snapshots/1.json
```

- **commit → snapshot**: `git show --name-only <hash>` reveals the `.cx/`
  file path, which encodes both the session ID and turn number.
- **snapshot → git status**: The current branch name `cx/<sid>`
  identifies the active session.
- **merge**: After `cx end --merge`, all metadata is on the base branch.

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Single binary, zero runtime deps |
| Git interaction | CLI subprocess | Avoids libgit2 compilation complexity |
| Storage | JSON + turn-indexed files | Zero deps, traceable by git history |
| Session identity | Git branch name | No global state file needed |
| .cx/ management | Tracked by git | Survives clone, no metadata loss |

## Build from Source

```bash
cargo install --path .
```

## License

MIT
