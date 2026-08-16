# Transcript-backed auto-titles

## Destination

Every supported agent-bearing tab takes its automatic title from its own matched
persisted session. A native provider title is displayed for free whenever one
exists; otherwise exactly one title is generated, from the first completed
human turn the operator actually submitted after binding. No screen change, spinner, clock, boot splash, resumed
history, or keystroke can cause a paid generation. Claude, Codex, OpenCode, Pi,
Kimi and Grok all reach this through one behavioral contract.

## Notes

- The settled specification is [spec.md](./spec.md). Read it before taking any
  ticket here; it is the authority on every rule these tickets implement. Its
  decisions were settled in a preceding grill, so this map opens with no planning
  tickets — the specification is the premise, not an output.
- The governing principle is that false negatives are preferred to false
  positives. A missing title is harmless; a false positive spends the operator's
  money or exposes the wrong transcript turn. Where a rule is ambiguous in
  practice, choose the cheaper, quieter failure.
- No title is a valid steady state. An agent with transcript persistence
  disabled or cleaned up is an expected unavailable condition, not an error the
  cockpit surfaces.
- The existing machine-wide auto-title toggle stays the only control. This work
  introduces no new setting, directory picker, provider-specific title options,
  manual title editor, or browser-side transcript protocol.
- Provider transcript stores are operational dependencies, not chartr-owned
  schemas (ADR 0002, agent-agnostic adapters). Adapters stay small, isolated and
  replaceable, and fail closed on a shape they do not recognize.
- Six agents already ship detection manifests, and both prompt delivery and
  headless generation recipes are existing per-adapter data tables. New
  per-adapter transcript knowledge belongs beside them as data, not as branches
  in the caller.
- Initial platform support is macOS and Linux. The cross-platform build must keep
  compiling with an unavailable transcript-process resolver rather than acquiring
  an implicit Unix dependency, exactly as the existing foreground-process seam
  does.
- `make test` runs the Go suite; `make vet` and `make check` cover vet and the
  web checks.

## Decisions so far

- [Foreground process identity and allowlisted state-root resolution](./tickets/01-process-identity-and-state-root-resolution.md) — `internal/proc` resolves a tab's foreground agent (or a chartr-launched pid) to pid/pgid, start time, working directory and a validated state root; the allowlist and defaults are pure per-adapter data in `internal/adapter/stateroot.go` (claude and kimi rows only; codex, opencode, pi and grok await ticket 04). Raw environments never leave the platform reader, ambiguity and unreadability resolve to unavailable, and platforms without a process reader compile to a reported-unavailable seam.

- [Normalized transcript events, the adapter contract, and the Claude adapter](./tickets/02-transcript-events-contract-and-claude-adapter.md) — `internal/transcript` owns discovery, binding, incremental reading and normalization behind two provider-neutral events (a native title, a completed top-level human turn) and one `Adapter`/`Session` contract a shared harness holds every provider to; Claude is the first implementation, binding through its `sessions/<pid>.json` process registry (working directory and session-start guard against pid reuse) or, failing that, working directory plus writes observed since the agent started, always unique or nothing. Native titles come from the store's own `ai-title` records, not the registry's session handle; cursors are byte offsets seated at the transcript's end at binding time, so history and a prompt persisted before binding stay behind them while a tab bound before its first write sees its opening turn; malformed, typeless and drifted records end a binding rather than being parsed on a guess.

## Not yet specified

- **Storage family of the five remaining providers.** Which of Codex, OpenCode,
  Pi, Kimi and Grok persist sessions as append-only JSONL and which use a
  database decides which reader shape each adapter in ticket 05 needs.
  <clears-with: 04>
- **Native-title and one-shot generation coverage per provider.** Whether each
  remaining provider exposes a usable native session title, a safe headless
  one-shot generation recipe, or neither. A provider with neither leaves its tabs
  permanently untitled, which is an accepted outcome that must be recorded rather
  than worked around. <clears-with: 04>

## Out of scope

- Windows foreground-process and process-environment discovery for transcript
  binding.
- Agent CLIs other than Claude, Codex, OpenCode, Pi, Kimi and Grok.
- Guessing from PTY bytes, browser key events, reconstructed screen changes, OSC
  titles, clocks, spinners or status counters when no transcript event exists.
- Paid generation from historical turns because a session was resumed,
  rediscovered or reattached.
- Cross-vendor title generation, or sending one provider's transcript through
  another provider.
- Reading hidden reasoning, system or developer instructions, tool calls, tool
  results, subagent transcripts, sidechains, summaries or full conversation
  history for title context.
- Editing, clearing or writing a provider's native session title or transcript.
- Persisting a duplicate transcript archive inside chartr.
- Guaranteeing compatibility with an unknown future private transcript schema;
  such a version fails closed until its fixture and parser are updated.
