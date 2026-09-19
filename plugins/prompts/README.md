# Saved Prompts

Saved Prompts is a bundled native Chartr plugin for a library of reusable prompts.
Open **Settings → Plugins → Saved Prompts** using its settings gear. Its table shows a title, a preview of
the prompt, and Copy, Edit, and Delete icon buttons with tooltips. Search matches both fields.
New prompt and Edit open a multiline editor modal above Settings; Cancel or
Escape discards the draft. Tab stays within the dialog, Enter inserts a new line
in the prompt, and Cmd+Enter (macOS) or Ctrl+Enter (Linux) saves. Delete opens a
confirmation modal naming the prompt. Validation and conflict errors stay in the
dialog. Copy puts only the full prompt text on the
clipboard, preserving whitespace and line breaks. The title is display metadata.

The library is shared across spaces and app windows through the singleton Settings window. It lives at
`$XDG_DATA_HOME/chartr/plugin-data/com.chartr.prompts/prompts.json` (by default
`~/.local/share/chartr/plugin-data/com.chartr.prompts/prompts.json`). Saves replace
the file atomically. Invalid or newer storage versions are reported and never
overwritten. Reload rereads the file after a manual repair or external edit.
Committed edits are reflected in consumers; stale drafts cannot
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

Saved Prompts also exports the common `PromptTemplates` service. Markdown Prompt
consumes those stable IDs alongside templates from other enabled plugins.
Only the saved body is expanded, without the title or added markup. The original
`com.chartr.prompts` ID and storage location are retained so existing libraries
continue to work. Old Saved Prompts tabs are retired on restoration; the library is
managed in plugin settings. This plugin does not write project files
or launch agents.

Successful saves, deletions and reloads notify template consumers. Markdown
Prompt refreshes its template palette; existing project files change only on its
next explicit **Apply changes → Save**.
