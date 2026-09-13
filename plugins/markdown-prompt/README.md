# Markdown Prompt

A bundled native plugin (`com.chartr.markdown-prompt`) for composing project
Markdown from editable text and live template references. Open it from the
surface picker in a folder space. It is independent of Saved Prompts: plain
text and any enabled template provider work without that plugin.

The template palette collects enabled `PromptTemplates` providers. The composer
is one continuous native text editor: template chips sit inline with ordinary
text and wrap with the paragraph. Click a preset to insert at the caret, or drag
it into the text; the caret follows the drop position. Drag an existing chip to
move it. Click its × button to remove it, or select the chip and use the normal
copy, cut, delete and undo commands. Removing a chip with × can also be undone.
The palette and inline templates share compact, borderless pills with a contrasting
background. Provider details are available on hover rather than in separate rows.

Text and template bodies concatenate verbatim, so the author controls spaces,
headings and line breaks. The left template list scrolls independently beside the
full-height prompt editor, with a visible, draggable scrollbar whenever it overflows. The refresh icon reloads the palette; providers also
refresh automatically when they report changes. **Preview**, above the prompt
editor, opens the expanded Markdown in a scrollable popup. Close it with **Close**,
Escape, or a click outside the dialog. Saving resolves references afresh. A
deleted template or disabled provider is marked unavailable and blocks writing
if the composition references it. Existing structured drafts load into the inline
editor without changing their composed output.

The native buffer encodes items as private, self-contained tokens rendered as
atomic fold placeholders. This preserves references through the editor's undo
stack and clipboard, including paste into another Markdown Prompt editor. The
configuration remains structured `Part` data and output files contain expanded
Markdown, never the editor tokens.

The fixed footer contains **Reset** and **Apply changes**. Reset restores the last
saved prompt and is disabled when there are no editor changes. It does not modify
project files. Apply changes opens a dialog containing the filename and destination folder.
**Save** saves the composition and applies its expanded Markdown. Edits and template
changes only reach project files on Save; opening a pane, refreshing templates,
closing a pane or restarting Chartr never writes project files. Unsaved editor
changes remain in the open pane until Save. **Cancel**, Escape or a click outside
the dialog discards filename edits. Validation errors stay in the dialog so the
filename can be corrected and Save retried.

Every Save follows the same file policy:

1. Validate a project-relative `.md` filename and resolve the prompt's templates.
2. Remove the previously applied marked section from its old file. Preserve
   surrounding content and delete the file if only whitespace remains.
3. If the expanded prompt is empty or whitespace-only, stop without touching the
   new destination.
4. Otherwise, update the marked section in the destination, append a marked
   section if it has none, or create the file with that section if it is missing.

Sections use `<!--chartr-markdown-prompt-begin-->` and
`<!--chartr-markdown-prompt-end-->`. When saving to the same file, the section is
replaced in place, preserving its position and surrounding text. Unchanged output
is not rewritten. Duplicate, reversed or incomplete markers block applying.
Destinations and marker structure are checked before cleanup, so validation
errors leave the last applied files intact. Unmarked user content is preserved.

The filename is relative to the current folder, for example `AGENTS.md`,
`CLAUDE.md` or `docs/agent-context.md`. Parent directories must exist when writing
new content. Traversal, symlink destinations and paths outside the project are
rejected. Writes use a staged file in the same directory and atomic replacement;
creation never clobbers a pre-existing file. Existing file permissions are retained.
Each file is written atomically; a save spanning multiple files is not a single
filesystem transaction.

Saved compositions live in the plugin's private data directory, keyed by project
path. The `.applied.json` sidecar tracks the previously applied destination for
cleanup across filename changes and restarts. There is no background file sync.
Provider and template IDs are persisted, so renaming a saved prompt preserves
references. Saving a stale pane reports a conflict; reopen the pane to load the
saved composition. Invalid or unsupported configuration files are reported and
never overwritten.

Older Enabled, Append and create-if-missing settings no longer control behavior.
Legacy New file outputs have no markers: their saved content receipts allow the
next Save to migrate or remove them only when they still match the last written
content. External edits to those legacy files block replacement until the original
content is restored or the generated section is explicitly marked.

The provider contract is documented in [plugin services](../../docs/plugins.md#prompt-template-providers).
