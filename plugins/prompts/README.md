# Prompts

Prompts is a bundled native Chartr plugin for a library of reusable prompts.
Open **Prompts** from the surface picker. Its table shows a title, a preview of
the prompt, and Copy, Edit, and Delete actions. Search matches both fields.
New prompt and Edit open a multiline editor; Cancel discards the draft, and
deletion requires a second click. Copy puts only the full prompt text on the
clipboard, preserving whitespace and line breaks. The title is display metadata.

The library is shared across spaces and panes. It lives at
`$XDG_DATA_HOME/chartr/plugin-data/com.chartr.prompts/prompts.json` (by default
`~/.local/share/chartr/plugin-data/com.chartr.prompts/prompts.json`). Saves replace
the file atomically. Invalid or newer storage versions are reported and never
overwritten. Reload rereads the file after a manual repair or external edit.
Edits made in another pane are reflected immediately; stale drafts cannot
silently overwrite or delete a changed record.

Each record has a stable `id`, `title`, and `prompt`. Renaming preserves its ID;
deleted IDs are not reused. Version 1 storage also tracks `next_id`:

```json
{
  "version": 1,
  "next_id": 2,
  "prompts": [
    { "id": "prompt-1", "title": "Code review", "prompt": "Review the current changes.\nExplain any defects." }
  ]
}
```

Future native consumers can depend on `com.chartr.prompts` and resolve the
`services::Prompts` service through `InstanceContext.services`. `list(cx)`
returns the current records; `resolve(id, cx)` resolves a stable reference.
Consumers inject only `SavedPrompt.prompt`, without the title or added markup.
The service reports unavailable or unreadable libraries as errors.

The table, storage module, and service contract are separate so a composer or
an AGENTS.md / CHARTR.md surface can reuse the library. Those surfaces and
automatic agent injection are not implemented here. Prompts has no dependency
on Agent or Skills and does not modify project files or start terminals.
