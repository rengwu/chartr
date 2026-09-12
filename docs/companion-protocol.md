# Companion protocol v1

TLS over TCP, default port 9847. UTF-8 newline-delimited JSON, one response per request in order. This development protocol accepts all clients without authentication. TLS uses a generated self-signed certificate; the mobile client does not verify it. No pairing code is needed. The optional legacy `token` field is ignored.

Request:

```json
{"version":1,"id":1,"operation":{"op":"list"}}
```

Response:

```json
{"id":1,"result":{"version":1,"spaces":[]},"error":null}
```

Errors have `result: null` and a human-readable `error` string. Unknown operations/fields and malformed or oversized framing close the connection. Unsupported versions return an error. Never retry a mutation after a transport failure: delivery may be ambiguous.

| Operation | Fields | Result |
| --- | --- | --- |
| `list` | none | `version`, `spaces: [{id,name,path,error,sessions:[{id,title,cwd,status,ended}]}]` |
| `screen` | `space`, `session` | `columns`, `rows`, `cells`, `cursor: [row,column]`, `show_cursor`, `app_cursor` |
| `watch` | `space`, `session`, `columns` (2–400), `rows` (1–240) | Same as `screen`; claims or renews mobile geometry for this connection |
| `release` | `space`, `session` | `true`; releases this connection’s mobile geometry lease |
| `submit` | `space`, `session`, `data` | `true`; bracketed paste and Enter in one operation |
| `history` | `space`, `session`, optional `snapshot`, `offset`, `known` | `text`, `snapshot`, `offset`, `next`, `total`; or `{unchanged:true,snapshot}` |
| `input` | `space`, `session`, `data` | `true`; raw UTF-8 input, no implicit Enter |
| `paste` | `space`, `session`, `data` | `true`; uses the host emulator’s bracketed-paste handling |
| `focus` | `space`, optional `session` | `true`; selects the space/session on desktop |
| `create` | `space` | `{accepted:true}`; asynchronous host-owned creation; refresh `list` for the resulting session or the space’s `error` |

Each cell is `[row,column,text,foreground,background,bold,underline]`. Colors are resolved `#rrggbb` values. Coordinates are zero-based within the terminal viewport. During a mobile lease this uses the mobile grid. `text` includes combining characters; wide-character spacer cells are omitted. Cursor coordinates may lie outside the viewport when the desktop is scrolled. Ignore those cursor positions. `app_cursor` selects `ESC O A/B/C/D` instead of `ESC [ A/B/C/D` for arrows.

History includes all text retained by Herdr, preserving blank lines and logical wrapping. Start with no `snapshot`; the result contains an immutable snapshot ID and up to 256 KiB of UTF-8 text. Fetch subsequent pages with that `snapshot` and the byte offset in `next`, until `next` is null. `total` is the full UTF-8 byte count. On a later refresh, send `known` with the last completed snapshot ID to avoid downloading unchanged text. Four snapshots are retained per window; if a continuation expires, restart from the first page. Paging does not move the desktop viewport. Herdr’s history settings still determine what the host retains; applications using an alternate screen expose their active screen, and manage their own internal history.

Limits: 64 KiB per request including its newline, 4 MiB per response, four connections, 16 queued host requests. Network reads/writes time out after five seconds; clients should poll `watch` around five times per second while a terminal is visible (also while reading history), or `list` periodically otherwise. Queued operations expire after four seconds. A `watch` lease lasts ten seconds and belongs to its TCP connection. Repeated watches renew it and update dimensions. A second connection receives an error if it tries to watch or write to an owned terminal. `release`, disconnect, shutdown, or expiry restores the prior desktop bounds; subsequent desktop layout can then resize normally. Desktop TerminalView is unmounted during a mobile lease and replaced with a read-only mirror, preventing competing resize events.

Sharing’s enabled state and bind address are persisted in the plugin data directory. The plugin shares its owning window. If several Chartr windows share, configure distinct ports. Disabling a plugin affects all catalogs through the existing global settings lifecycle; every associated listener is stopped.
