---
type: task
blocked_by: [01]
---

# Reskin the leaf surfaces — dialogs, forms, detail, payload

## Question

Rebuild the self-contained surfaces — the ones that don't own the shell layout — onto shadcn-svelte primitives, tokens, IBM Plex, and Phosphor icons, deleting their bespoke `app.css` rules as each lands. In scope: `Modal.svelte` → shadcn **Dialog** (the `RegisterForm` "Add a space" dialog rides on it); `RegisterForm.svelte` → **Input** + **Button** + inline validation (the `.register-*` block); `DetailPane.svelte` → **Card**/**ScrollArea** with **Badge** status pills (`.dp-*`, the rendered-markdown `.dp-md` styles rehomed under a `prose`-like token scale); `PayloadPreview.svelte` → the wide Dialog + **ScrollArea** with per-segment layer provenance as **Badge**s (`.modal-card.wide`, `.payload-*`).

Apply distill on the way through: the hand-rolled `.btn` / `.icon-btn` / `.kind-btn` / `.field-src` variants collapse into the shared Button/Badge primitives with a small, named set of variants — not a bespoke class per site. Emoji/unicode glyphs (× close, ▲ warn, ⚠, the register states) become Phosphor icons. Status colour keeps its meaning: `resolved` → a success-tinted badge, `proposed`/`claimed` → neutral/`--primary` emphasis, `out_of_scope` → muted, problems → `--destructive` — no amber. Every surface reads one primary action.

The behaviour is fixed (map "Out of scope"): the dialog open/close contract, the detail pane's inline-blocker assembly and deep-link selection, and the payload's layer provenance all work exactly as before — only the skin changes. Keep the `Modal`/`Dialog` API compatible with its callers or update them in the same ticket.

Done when: each listed surface renders on shadcn-svelte primitives with tokens, IBM Plex, and Phosphor icons, and its old `app.css` block is deleted; the register/add-space, detail-pane, and payload-preview flows behave identically (open, validate, select, scroll, dismiss); no amber remains on these surfaces; `svelte-check` is clean and the Vite build, `vitest`, `go vet`, and `go test` pass.