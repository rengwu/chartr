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
headings and line breaks. The expanded preview shows the output and refreshes
when providers report changes. Apply resolves all references afresh and activates
automatic synchronization of that composition. A
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
- **Append** requires an existing UTF-8 Markdown file. The first Apply appends a
  section; subsequent applies replace only the content between
  `<!--chartr-markdown-prompt-begin-->` and
  `<!--chartr-markdown-prompt-end-->`. Text outside the section remains byte for
  byte intact. Duplicate, reversed or incomplete markers block writing.
- **Enabled** gates Apply. Disabling does not erase previously written content.
  **Save draft** persists configuration without applying draft text or template
  changes. Saving the disabled state pauses the last applied composition;
  saving Enabled resumes it. Apply activates the current draft and destination.

The filename is relative to the current folder, for example `AGENTS.md`,
`CLAUDE.md` or `docs/agent-context.md`. Parent directories must exist. Traversal,
symlink destinations and paths outside the project are rejected. Writes use a
staged file in the same directory and an atomic replacement; creation never
clobbers a pre-existing file. Existing file permissions are retained.

Drafts and applied compositions live separately in the plugin's private data
directory, keyed by project path. The `.applied.json` sidecar records the project,
last applied composition and current ownership receipts. Background updates do
not alter draft files or invalidate open drafts. Provider and template IDs are persisted, so renaming a
saved prompt preserves references. Applying or saving a stale pane reports a
conflict; reopen the pane to load the saved composition. Invalid or unsupported
configuration files are reported and never overwritten.

The provider contract is documented in [plugin services](../../docs/plugins.md#prompt-template-providers).

## Automatic synchronization

After Apply, a plugin-lifetime worker follows the composition's referenced
providers even when its pane is closed. Saved Prompts and Skills publish change
notifications after successful changes; these trigger immediate re-expansion.
A two-second reconciliation also discovers external local-source changes,
provider disablement and newly applied compositions from another instance.
Saved applied compositions resume when Chartr starts and Markdown Prompt is
enabled. No background synchronization runs while Chartr is closed or the plugin
is disabled. Older drafts become active when applied with this version.

Only the last applied parts, filename and mode are used. Editing or saving an
unapplied draft does not change the active composition. Unchanged output is not
rewritten. New-file ownership checks and append-section boundaries apply equally
to automatic updates. Source changes during expansion supersede older results;
explicit Apply and pause operations win over an in-flight refresh. A missing
provider, invalid template or external edit leaves the last good file untouched
and reports a synchronization error through plugin background status and the
open composer. Errors are retried on subsequent changes/reconciliation.
