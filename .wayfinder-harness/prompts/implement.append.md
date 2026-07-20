## Frontend work follows the design system

If this ticket touches `web/`, the cockpit chrome runs on a real design system —
**shadcn-svelte + Tailwind v4** on an olive/neutral token theme (ADR 0012).
**Read `docs/design-system.md` before writing any UI** and follow it: style on
**tokens + primitives + Phosphor**. Do not hand-roll chrome CSS or a `.btn`, do
not write a raw hex, do not re-introduce amber into the chrome (`--destructive` is
the only chroma), and do not reach inside an island's renderer to re-theme it —
go through the seam (ADR 0010). If a needed colour has no token, flag the missing
role rather than inlining one.
