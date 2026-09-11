# Markdown Prompt

A bundled native plugin (`com.chartr.markdown-prompt`) for composing project
Markdown from editable text and live template references. Open it from the
surface picker in a folder space. It is independent of Saved Prompts: plain
text and any enabled template provider work without that plugin.

The template palette collects enabled `PromptTemplates` providers. The composer
is one continuous native text editor: template chips sit inline with ordinary
text and wrap with the paragraph. Click a preset to insert at the caret, or drag
it into the text; the caret follows the drop position. Drag an existing chip to
move it. Click a chip to select it, then use the normal copy, cut, delete and undo
commands. Provider details are available on hover rather than in separate rows.

Text and template bodies concatenate verbatim, so the author controls spaces,
headings and line breaks. **Refresh** reloads the template palette; providers also
refresh automatically when they report changes. **Preview**, below the contents
editor, opens the expanded Markdown in a scrollable popup. Close it with **Close**,
Escape, or a click outside the dialog. Live updates resolve references afresh. A
deleted template or disabled provider is marked unavailable and blocks writing
if the composition references it. Existing structured drafts load into the inline
editor without changing their composed output.

The native buffer encodes items as private, self-contained tokens rendered as
atomic fold placeholders. This preserves references through the editor's undo
stack and clipboard, including paste into another Markdown Prompt editor. The
configuration remains structured `Part` data and output files contain expanded
Markdown, never the editor tokens.

- **New file** creates and then maintains the entire named Markdown file. The
  plugin records its last written content. It updates an existing file only if
  that receipt matches; unrelated files and external edits are never silently
  overwritten. Choose another filename or use Append when a conflict is shown.
- **Append** uses an existing UTF-8 Markdown file unless **Create file if it
  doesn't exist** is enabled (off by default). With that switch enabled, a missing
  file is created with the marked section. The first update appends a
  section; subsequent updates replace only the content between
  `<!--chartr-markdown-prompt-begin-->` and
  `<!--chartr-markdown-prompt-end-->`. Text outside the section remains byte for
  byte intact. Duplicate, reversed or incomplete markers block writing.
- **Enabled** controls live file updates. Switching it on saves and activates the
  current composition immediately. Switching it off pauses updates and preserves
  previously written content. Text and template references autosave after a
  500 ms editing pause. While enabled, those saved edits also update the file;
  while disabled, they remain a draft. The status beside the switch reports
  saving, file updates or errors. Opening a fresh, untouched composer does not
  create a project file.

**Mode** opens a modal containing the output mode, its description, the
create-if-missing switch, the filename and the destination folder. **Save**
commits these settings together and updates the file when Enabled is on.
**Cancel**, Escape or a click outside the modal discards the modal edits.
Text autosave and live sync continue to use the saved settings until Save is
clicked.

The filename is relative to the current folder, for example `AGENTS.md`,
`CLAUDE.md` or `docs/agent-context.md`. Parent directories must exist. Traversal,
symlink destinations and paths outside the project are rejected. Writes use a
staged file in the same directory and an atomic replacement; creation never
clobbers a pre-existing file. Existing file permissions are retained.

Drafts and applied compositions live separately in the plugin's private data
directory, keyed by project path. The `.applied.json` sidecar records the project,
live composition and current ownership receipts. Background updates do not alter
saved editor settings or invalidate open drafts. Provider and template IDs are
persisted, so renaming a saved prompt preserves references. Autosaving a stale pane reports a
conflict; reopen the pane to load the saved composition. Invalid or unsupported
configuration files are reported and never overwritten.

The provider contract is documented in [plugin services](../../docs/plugins.md#prompt-template-providers).

## Automatic synchronization

While Enabled is on, a plugin-lifetime worker follows the composition's referenced
providers even when its pane is closed. Saved Prompts and Skills publish change
notifications after successful changes; these trigger immediate re-expansion.
A two-second reconciliation also discovers external local-source changes,
provider disablement and newly enabled compositions from another instance.
Saved live compositions resume when Chartr starts and Markdown Prompt is
enabled. No background synchronization runs while Chartr is closed or the plugin
is disabled. Opening an older saved draft adopts its current Enabled state.

Live updates use the latest autosaved enabled composition. The create-if-missing
setting also allows sync to recreate a missing file with its marked section.
Pending edits pause older sync work until autosave finishes. Closing the pane
allows its pending autosave to finish; disabled drafts leave the output untouched.
Unchanged output is not rewritten. New-file ownership checks and append-section
boundaries apply equally to automatic updates. Source changes during expansion
supersede older results. A missing provider, invalid template, invalid filename or
external edit leaves the last good file untouched, while the editor draft is
still saved. Errors appear in plugin background status and the open composer and
are retried on subsequent changes/reconciliation.
