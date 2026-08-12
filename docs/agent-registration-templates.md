# Agent registration templates

This is a non-exhaustive list. We highly welcome public updates here. If you notice something is not working or missing, feel free to open a pull request.

## Claude Code, Claude Opus 4.8

```toml
[agents.claude-opus-4-8]
adapter = "claude"
args = ["--model", "claude-opus-4-8", "--dangerously-skip-permissions"]
env = ["CLAUDE_CONFIG_DIR=~/.claude"]
```
