# Agent registration templates

This is a non-exhaustive list. We highly welcome public updates here. If you notice something is not working or missing, feel free to open a pull request.

## Claude Code, Claude Opus 4.8

```toml
[agents.claude-opus-4-8]
adapter = "claude"
args = ["--model", "claude-opus-4-8", "--dangerously-skip-permissions"]
env = ["CLAUDE_CONFIG_DIR=~/.claude"]
```

## Kimi Code

```toml
[agents.kimi-k3]
adapter = "kimi"
args = ["-m", "kimi-code/k3", "--yolo"]
```

Kimi gates startup on a "Trust this folder?" dialog whose default, "Don't
trust", exits the process — a spawned session in a folder kimi has never seen
trusted dies before its first turn. chartr records kimi's workspace-trust
marker for the space before every launch (ADR 0002, launch preflight), so the
dialog never appears. If the binding relocates kimi's data directory, write it
in `env` (e.g. `env = ["KIMI_CODE_HOME=~/.kimi-code-work"]`) and the marker
follows it.
