# Transcript-backed auto-titles

## Destination

Every supported agent-bearing tab takes its automatic title from its own matched
persisted session. A native provider title is displayed for free whenever one
exists; otherwise exactly one title is generated, from the first completed
human turn the operator actually submitted after binding. No screen change, spinner, clock, boot splash, resumed
history, or keystroke can cause a paid generation. Claude, Codex, Pi, Kimi and
Grok all reach this through one behavioral contract. OpenCode remains a normal
supported agent, but this minor feature deliberately does not read its database.

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

- [Foreground process identity and allowlisted state-root resolution](./tickets/01-process-identity-and-state-root-resolution.md) — `internal/proc` resolves a tab's foreground agent (or a chartr-launched pid) to pid/pgid, start time, working directory and a validated state root; the allowlist and defaults are pure per-adapter data in `internal/adapter/stateroot.go`. Raw environments never leave the platform reader, ambiguity and unreadability resolve to unavailable, and platforms without a process reader compile to a reported-unavailable seam. OpenCode has no row because it has no transcript adapter.

- [Normalized transcript events, the adapter contract, and the Claude adapter](./tickets/02-transcript-events-contract-and-claude-adapter.md) — `internal/transcript` owns discovery, binding, incremental reading and normalization behind two provider-neutral events (a native title, a completed top-level human turn) and one `Adapter`/`Session` contract a shared harness holds every provider to; Claude is the first implementation, binding through its `sessions/<pid>.json` process registry (working directory and session-start guard against pid reuse) or, failing that, working directory plus writes observed since the agent started, always unique or nothing. Native titles come from the store's own `ai-title` records, not the registry's session handle; cursors are byte offsets seated at the transcript's end at binding time, so history and a prompt persisted before binding stay behind them while a tab bound before its first write sees its opening turn; malformed, typeless and drifted records end a binding rather than being parsed on a guess.

- [Transcript stores for Codex, OpenCode, Pi, Kimi and Grok](./tickets/04-research-remaining-provider-transcript-stores.md) — research found four append-only JSONL stores and one outlier: OpenCode's live SQLite/WAL database. The four file-backed providers proceed through the shared tail; OpenCode is deliberately omitted because owning a SQLite reader is disproportionate infrastructure for this minor feature. Measurements remain in [assets/provider-transcript-stores.md](./assets/provider-transcript-stores.md).

- [Transcript-driven titles replace the screen-derived gate](./tickets/03-transcript-driven-titles-replace-the-screen-gate.md) — the behavior change is wired: the worked-gate, startup grace, unchanged-screen guard, debounce and screen-tail context are deleted, and a tab now folds its own session's normalized events on the sampler's slow tick. A native title publishes immediately, refreshes with no debounce, is normalized to the shared one-line `MaxTitleRunes` contract and blocks all spending; otherwise the first completed turn schedules one generation, spent at *scheduling* time so a decline, an unusable answer, a cancellation or an exhausted ladder ends it forever. The generator receives only `TitleRequest{Adapter, Env, Context}` — the turn's prompt then its final answer inside the existing 1500-rune budget, shared so neither side can eat the other, plus `adapter.StateRootEnv` for the live process's own root, which makes generation same-profile as well as same-adapter. Title state is keyed by (adapter, pid), so an agent change drops the binding, the spent attempt and the title together, while a dead tab keeps its title and stops being observed; the toggle off drops the binding and re-enabling re-binds at the transcript's end. Tests drive the real manager through an injected transcript source and generator.

- [Remaining transcript adapters](./tickets/05-remaining-transcript-adapters.md) — Codex, Pi, Kimi and Grok implement the same JSONL contract as Claude. OpenCode's completed database adapter and the bespoke SQLite/WAL reader were removed after implementation review: no title is the cheaper outcome than carrying a database subsystem solely for this convenience feature.

## Settled limitation

- Codex, Pi, Kimi and Grok create their stores on first submit. The cursor still
  seats at the end when the store appears, so their opening turn stays historical
  and the second turn is the first paid-title opportunity. A one-turn session may
  remain untitled; that false negative is intentional.

## Out of scope

- Windows foreground-process and process-environment discovery for transcript
  binding.
- Transcript-backed titles for OpenCode or agent CLIs other than Claude, Codex,
  Pi, Kimi and Grok.
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
