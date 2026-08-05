---
type: grilling
blocked_by: [01, 04]
---

# The two core payloads

## Question

chartr composes one payload today: core + role skill + a context bundle (`internal/prompt/compose.go`). It now composes two shapes, and neither is what it composes today.

The **free-session** payload is the new artifact and the harder one, because it is defined by a restraint: it tells the agent what chartr *is* and what skills exist, and nothing that would change how the agent behaves. The **ticket** payload keeps today's `core` body, but the role skill it used to concatenate now resolves out of a registered source, and the context bundle it carries sourced its glossary from a skill that may no longer exist.

Settle:

- **What "capabilities" means, concretely.** The free-session payload names the wayfinder map, the tickets, the folder structure. Is that a static paragraph, or does it carry live facts — this space's path, whether a map exists, which maps exist, the frontier? Live facts make the session useful immediately; they also make the payload a second reader of the map with its own staleness. Where is the line, and does an empty space (no `.plan/`) get a different payload from a space mid-effort?
- **How sources appear in it.** Name and location only, so the agent can lazily look one up. Does it list the *skills* discovered in each source too, or only the sources? Listing skills is what makes "use Matt Pocock's prototype skill" resolvable without a filesystem walk mid-turn; not listing them is what keeps the payload from growing with the operator's library. Where a source is git-backed, is the URL shown or only the local cache path?
- **The restraint, stated as a test.** "Don't provide any more instructions that may change the agent's behavior" needs an operational form, because the conventions pointer and the sources list are both, strictly, instructions. What is the rule that admits those two and rejects the next thing someone wants to add?
- **What the ticket payload keeps.** Today's `core` stays. Does the role skill's body still get concatenated ahead of the context bundle — now read from the resolved source — or does the payload point at it the way it points at the conventions? Concatenating preserves the guarantee that the role prompt was actually read; pointing keeps the payload small and treats sourced skills uniformly.
- **The context bundle's glossary.** It is inlined today from `tracker-convention`'s supporting file. If the conventions ruleset absorbs that vocabulary, the bundle sources it from the ruleset instead — or stops carrying it, because the payload already points at the ruleset. Decide, and say what `Bundle` and the `ctxPart` list look like afterwards.
- **The payload preview.** The cockpit renders a payload with per-segment provenance (`Segment.Layer`). Layers are gone; segments now come from sources, the embedded cores, and the bundle. What provenance does the preview show, and does the free-session payload get a preview at all — there is no ticket to preview it against.

## Done when

Both payloads are specified as concrete documents — what each section is, where its text comes from, and what varies per space or per ticket — together with the restraint rule for the free-session payload, the ruling on concatenate-versus-point for role bodies, and what becomes of the glossary part and the provenance the preview renders.
