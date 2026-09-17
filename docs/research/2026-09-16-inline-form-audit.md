# Inline form audit — 16 September 2026

Reviewed the current workspace's host UI and bundled plugins, including native
text inputs, editors, HTML forms, and Save/Cancel flows. This is a source review;
third-party installed plugins are outside its scope.

## Implemented

**Settings → Plugins → Install from Git** now uses the shared dialog surface
above the Settings window instead of inserting a form into the plugin list.
Opening it focuses the URL. Tab and Shift-Tab stay within the dialog; Enter
submits, and Escape, Cancel, or clicking outside dismisses it and restores focus
to the trigger. Empty input shows its error inside the dialog. A nonempty URL
closes the dialog and starts the existing inspection, trust confirmation, and
installation flow. The URL remains available if the user needs to reopen it.

Source: [plugin settings UI](../../crates/chartr/src/settings_window/plugins.rs),
[Settings window](../../crates/chartr/src/settings_window.rs).

## Follow-up changes

| Surface | Finding | Resolution |
| --- | --- | --- |
| Saved Prompts → Delete | Inline **Delete? / Cancel** confirmation. | Replaced with a native modal naming the prompt, with Cancel/Delete actions. Conflicts keep the modal open. |
| Inbox conversation rename | Dormant input plus Save/Cancel row above the terminal. | Removed the form, state, save handler, and storage rename API. Previously stored titles remain readable. |
| Saved Prompts → New/Edit | A dedicated editor replaced the library. | Replaced with a native modal above Settings. The body scrolls while the header and actions stay visible. |

Saved Prompts now uses Copy/Edit/Delete icon buttons with tooltips and a copied
checkmark. Skill sources uses an Edit icon; its up/down buttons were removed,
with drag-and-drop ordering retained.

Sources:

- [Saved Prompts](../../plugins/prompts/src/lib.rs): `PromptsView::table`,
  `render_settings`, and the [prompt dialogs](../../plugins/prompts/src/dialog.rs).
- [Conversation view](../../crates/chartr/src/conversations/view.rs) and
  [state](../../crates/chartr/src/conversations.rs).
- [Existing rename dialogs](../../crates/chartr/src/app/rename.rs).

Apart from the Git install form addressed here, this review found no other
active inline data-entry form that clearly requires a modal conversion.

## Appropriate as currently presented

- Agent registration/editing and deletion, Skill source registration/editing
  and deletion, Markdown Prompt apply/preview, and Space/Group renaming already
  use dialogs. Wayfinder's final launch review uses an HTML modal dialog.
- Wayfinder's launch options belong beside the selected ticket; the Agent
  launch composer is its pane's primary task.
- Companion's address and Start/Stop sharing controls are persistent service
  settings. General preferences, declarative plugin settings, search fields,
  and the browser address bar should remain directly accessible.

## Verification

`cargo check -p chartr --locked` passed. All seven Settings window tests passed,
including a modal interaction test covering layout at 720 × 420, unchanged
page layout, empty submission, focus cycling, Cancel/Enter, Escape, and backdrop
dismissal without activating the sidebar beneath it. No plugin was installed
as part of verification.

Follow-up validation: all five Saved Prompts tests, four Inbox tests, and 30
conversation storage/transcript tests passed (two environment-dependent tests
were ignored). Twelve of thirteen Skill source tests passed, including both
drag-and-drop tests. The unchanged `sources.rs` has an existing failure in
`templates_keep_the_combined_id_and_scope_each_enabled_source`: its expected
Markdown heading differs from the formatter's current heading. No template
formatting was changed in this work.
