---
name: cx-workflow
description: "Automatically use `cx` (AI-native code management CLI) to track AI coding sessions in git-based projects. When Codex is performing a coding task in a git repository, integrate `cx` into the workflow: initialize if needed, start a session before coding, create snapshots after changes, and end the session on completion. Use with the developer-engineer skill for any coding task where cx session tracking is desired."
---

# cx Workflow

When Codex is performing a coding task in a git repository, automatically integrate `cx` into the workflow.
`cx` is a git companion that creates tracking branches for each AI session and records each change as a snapshot with semantic intent labels.
If the repository does not have `.cx/` initialized, `cx start` will initialize it automatically.

## Workflow

When starting a new task, run:

```
cx start "<prompt>"
```

The `prompt` should briefly summarize the task objective, for example
`"Add payment module supporting Alipay and WeChat Pay"` or `"Fix login redirect bug"`.

During coding, after each meaningful set of changes, run:

```
git add <files>
cx apply -m "<message>" --intent <type>,scope=<label>[,risk=<level>]
```

- `-m`: Description of the changes made
- `--intent`: Semantic label; see Intents reference below

When the task is complete, end the session:

```
cx end --merge
```

This merges the session branch back into the base branch, including all `.cx/` metadata.

If the task is interrupted or the session should be discarded, use:

```
cx end --abandon
```

### Workflow Summary

| Phase | Command | Description |
|-------|---------|-------------|
| Start | `cx start "<prompt>"` | Auto-initializes `.cx/` if needed, then creates a session branch |
| Code | `git add + cx apply ...` | Record snapshots with semantic labels |
| Finish | `cx end --merge` | Merge into base branch |
| Discard | `cx end --abandon` | Discard the session branch |

## Intents Reference

```
--intent <type>,scope=<scope>[,risk=<level>]
```

### Type (required)
- `feature` — New feature
- `fix` — Bug fix
- `refactor` — Code reorganization
- `style` — UI/CSS changes
- `docs` — Documentation changes
- `dependency` — Dependency changes
- `test` — Tests
- `chore` — Miscellaneous

### Scope (optional)
Arbitrary label, e.g. `payment`, `login`, `auth`, `ui`.

### Risk (optional, defaults to low)
- `low` — Low risk change
- `medium` — Medium risk
- `high` — High risk

## Example

Task: "Add payment module supporting Alipay and WeChat Pay"

```
cx start "Add payment module supporting Alipay and WeChat Pay"

# After writing Alipay signature logic
git add pay.rs
cx apply -m "Add Alipay signature verification" --intent feature,scope=payment

# After extracting utility function
git add crypto.rs
cx apply -m "Extract signature helper" --intent refactor,scope=crypto,risk=medium

# Task complete
cx end --merge
```

## Notes

- `cx` must be on PATH. If `cx` is unavailable, install it first (see Building section below)
- `cx start` automatically initializes `.cx/` if the repository does not have it yet — no manual `cx init` needed
- Check `cx status` before `cx start` to ensure no session is already active
- `cx apply` must be run after `git add`; it commits staged changes along with metadata
- A new session can only be created after `cx end --merge`
- **Do not run `git merge` manually after `cx end --merge`** — this will conflict with `cx`'s metadata structure. Always use `cx end --merge` to finalize.
- This skill only applies to git repositories. If the current directory is not a git repo, `cx` will not work and should be skipped.

## Building and Installing cx

If `cx` is not yet installed on the target machine, install it via:

```bash
curl -fsSL https://raw.githubusercontent.com/junf108/cx/main/scripts/install.sh | sh
```

Or build from source:

```
cd /path/to/cx
cargo install --path .
```

Verify installation with `which cx`.
