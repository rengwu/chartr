// Operator actions are plain HTTP request/response (ADR 0010): a failed action
// surfaces as a response the caller shows, never as a silent state change. The
// resulting state arrives separately over the control socket as a fresh whole
// snapshot, so these helpers return only the action's own result (or throw).

// ActionError carries chartr's own message for a refused action, and — for the
// refusals chartr tags — a stable `code` the surface can branch on. Only one
// refusal is tagged today (LIVE_SESSION), because only one is the operator's to
// overrule; everything else stays a message to show.
export class ActionError extends Error {
  readonly code?: string
  constructor(message: string, code?: string) {
    super(message)
    this.code = code
  }
}

// LIVE_SESSION marks the one refusal a confirmation can turn into a spawn: the
// space already has a live session (ADR 0003 as amended). Every other 409 — a held
// ticket, a missing agent — is a refusal of fact, so the surface must not offer to
// override it, and matching on the code rather than the message is what keeps the
// copy editable without silently re-classifying a refusal.
export const LIVE_SESSION = 'live_session_exists'

async function send(method: string, path: string, body?: unknown): Promise<unknown> {
  // Every POST declares JSON, and so does anything carrying a body — the server
  // refuses an undeclared write with a 415 (websocket-origin-fix, ticket 03).
  // That includes the actions that send no body at all (open a shell, pick a
  // folder): a POST is the one write a browser will
  // send cross-origin with no preflight, and the declaration is what puts one
  // behind a preflight the server never answers.
  const declaresJson = method === 'POST' || body !== undefined
  const res = await fetch(path, {
    method,
    headers: declaresJson ? { 'Content-Type': 'application/json' } : undefined,
    body: body === undefined ? undefined : JSON.stringify(body),
  })
  if (!res.ok) {
    let msg = `${method} ${path} failed (${res.status})`
    let code: string | undefined
    try {
      const data = (await res.json()) as { error?: string; code?: string }
      if (data?.error) msg = data.error
      code = data?.code
    } catch {
      // Non-JSON error body; keep the status-line message.
    }
    throw new ActionError(msg, code)
  }
  if (res.status === 204) return null
  const text = await res.text()
  return text ? JSON.parse(text) : null
}

export interface RegisterResult {
  id: string
  path: string
  gitInited: boolean
}

export function registerSpace(path: string): Promise<RegisterResult> {
  return send('POST', '/api/spaces', { path }) as Promise<RegisterResult>
}

// pickFolder raises the operator's own OS folder chooser and resolves to the
// folder they named, or `cancelled` when they dismiss it. It registers nothing —
// the caller posts the returned path through registerSpace — so the announced
// `git init` and every refusal keep the one response shape they already have.
//
// The request stays open for as long as the dialog does, which is exactly the
// point: the chooser is modal to the operator, not to the page.
export interface PickResult {
  path?: string
  cancelled: boolean
}

export function pickFolder(): Promise<PickResult> {
  return send('POST', '/api/spaces/pick') as Promise<PickResult>
}

export function deregisterSpace(id: string): Promise<void> {
  return send('DELETE', `/api/spaces/${encodeURIComponent(id)}`) as Promise<void>
}

// Open a registered space in the platform's folder browser. The client sends
// only the registry id; the server resolves the trusted path, so this control
// cannot be turned into an arbitrary local-path opener.
export function openSpaceFolder(id: string): Promise<void> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/open`) as Promise<void>
}

// Reconcile this space's `.chartr/skills` mirror against the enabled sources
// now, rather than waiting on the next spawn barrier — the manual prewarm behind
// the cockpit's sync control. Old skills are pruned and current ones (re)written
// server-side; the refreshed state rides back on the control snapshot.
export function syncSkills(id: string): Promise<void> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/skills/sync`) as Promise<void>
}

// reorderSpaces sets the sidebar arrangement: the *complete* ordered list of
// space ids, never a per-row move. One endpoint takes the whole list, so the
// pointer drag and the keyboard share a single write path; the write is
// idempotent, and two drags in quick succession cannot interleave into a
// half-moved sidebar. A list that omits a registered space, names an unknown id,
// or repeats one is refused with a 400 that changes nothing — a partial list is
// a client bug, not a partial reorder. The new order arrives back as a fresh
// snapshot over the control socket like every other action's result.
export function reorderSpaces(ids: string[]): Promise<void> {
  return send('POST', '/api/spaces/reorder', { ids }) as Promise<void>
}

// setSpaceAgent records the agent a space should spawn with, without spawning —
// the action footer's selector persists the operator's pick immediately, so it
// reads as a saved setting rather than a choice that only sticks once they launch.
// The remembered agent rides the next model snapshot as `space.lastAgent`.
export function setSpaceAgent(id: string, agent: string): Promise<void> {
  return send('PUT', `/api/spaces/${encodeURIComponent(id)}/agent`, { agent }) as Promise<void>
}

// renameSpace sets the operator's own display name for a space, persisted per-path
// in the registry. It is presentation only — nothing on disk is renamed, the path
// and id are untouched — so it reads as a saved label the moment it is set. An
// empty name clears the override, and the space reads by its folder basename
// again. The new label rides the next model snapshot as `space.name`.
export function renameSpace(id: string, name: string): Promise<void> {
  return send('PUT', `/api/spaces/${encodeURIComponent(id)}/name`, { name }) as Promise<void>
}

// openTerminal opens an ad-hoc shell in the space's working tree (story 29) and
// returns its terminal id — the key the terminal socket attaches by. The new tab
// also arrives over the control socket.
export function openTerminal(id: string): Promise<{ id: string }> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/terminals`) as Promise<{
    id: string
  }>
}

// closeTerminal ends an ad-hoc shell on the operator's command — ad-hoc shells
// have no lifecycle and are ended only by the human. The tab drops from the next
// snapshot.
export function closeTerminal(spaceId: string, termId: string): Promise<void> {
  return send(
    'DELETE',
    `/api/spaces/${encodeURIComponent(spaceId)}/terminals/${encodeURIComponent(termId)}`,
  ) as Promise<void>
}

// markTerminalSeen acknowledges a run that finished while the operator was
// elsewhere: the tab is in front of them now, so its dot clears in the snapshot
// (session-notifications). Focusing is the whole acknowledgement — there is no
// dismiss and no clear-all — and posting for a tab that carries no dot is an
// ordinary no-op, which is why the caller may post on focus without checking.
export function markTerminalSeen(spaceId: string, termId: string): Promise<void> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(spaceId)}/terminals/${encodeURIComponent(termId)}/seen`,
  ) as Promise<void>
}

// launchFree starts a **free session** in a space on a chosen agent (skill-sources
// ticket 08): a live, ticketless tab told the free payload — what chartr is and
// what skills exist, and nothing about how to behave. It shares only the adapter's
// spawn primitive with a real session (no map or ticket, no claim, no lifecycle,
// ended only by the human, exactly like an ad-hoc shell) and returns the new tab's
// id. It names its agent like every other spawn (agent-selection ticket 03): there
// is no role behind it to fall back on, so the name is always sent. There is
// nothing else to send — a free session takes no skill and no context line.
export function launchFree(id: string, agent: string): Promise<{ id: string }> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/launch`, {
    agent,
  }) as Promise<{ id: string }>
}

// previewPayload composes what a session for one ticket and role would be told
// (ticket 08) — the resolved skills and the context bundle — with per-part
// provenance. Read-only inspection, so a GET; the
// chartr reads the library and the map fresh, so an edit on disk shows up here.
export function previewPayload(
  id: string,
  slug: string,
  num: number,
  role: string,
): Promise<import('./model').Payload> {
  return send(
    'GET',
    `/api/spaces/${encodeURIComponent(id)}/maps/${encodeURIComponent(slug)}/tickets/${num}/payload?role=${encodeURIComponent(role)}`,
  ) as Promise<import('./model').Payload>
}

// previewFreePayload composes what a *free* session is told: the same seam, the
// same modal, four parts, and no ticket or role to choose. It names no space —
// a free payload holds no live fact about one, which is why it hangs off the
// settings surface rather than a space card.
export function previewFreePayload(): Promise<import('./model').Payload> {
  return send('GET', '/api/payload/free') as Promise<import('./model').Payload>
}

// The skill-source list's six actions. Each returns only its own result; the new
// list arrives over the control socket as a fresh snapshot, so nothing here is
// applied optimistically and a refusal leaves the rendered list untouched.

// registerSource appends a folder or a git repository of skills to the list. The
// kind is declared, never inferred from what was typed. Registering a `git`
// source with git absent from PATH is refused at the gate, naming why, before a
// row is written — so the thrown message is the whole story.
export function registerSource(src: {
  name: string
  kind: 'dir' | 'git'
  path?: string
  url?: string
  ref?: string
}): Promise<{ name: string; path: string }> {
  return send('POST', '/api/config/sources', src) as Promise<{ name: string; path: string }>
}

// removeSource drops a row and touches nothing it pointed at: a dir source is
// the operator's own folder, and a git checkout under `sources/` is left where
// it is — chartr does not collect them.
export function removeSource(name: string): Promise<unknown> {
  return send('DELETE', `/api/config/sources/${encodeURIComponent(name)}`)
}

// setSourceEnabled toggles a source off without losing it or its position.
export function setSourceEnabled(name: string, enabled: boolean): Promise<unknown> {
  return send('PUT', `/api/config/sources/${encodeURIComponent(name)}/enabled`, { enabled })
}

// reorderSources rewrites the list order, which *is* resolution order. The whole
// list goes every time; the server refuses anything that is not a permutation of
// what it holds, so a stale view cannot half-apply a move.
export function reorderSources(names: string[]): Promise<unknown> {
  return send('POST', '/api/config/sources/reorder', { names })
}

// refreshSource moves a git source to the tip of its ref and records the new
// pin. It resets chartr's checkout: local edits inside it are discarded, which
// is why the row says so beside the action.
export function refreshSource(name: string): Promise<{ name: string; commit: string; fetched: string }> {
  return send('POST', `/api/config/sources/${encodeURIComponent(name)}/refresh`) as Promise<{
    name: string
    commit: string
    fetched: string
  }>
}

// SpawnResult is the spawn action's own response — the session it started, the
// resolved agent and args, and the payload hash the claim commit recorded. The
// live session tab arrives separately over the control socket.
export interface SpawnResult {
  sessionId: string
  ticketNum: number
  role: string
  agent: string
  args?: string[]
  payloadSha: string
}

// spawnSession spawns a session on a frontier ticket (ticket 09): chartr
// writes the claim commit, composes and archives the payload, and launches the
// chosen agent's TUI with the read-this-file opener typed in. A blocked spawn — an
// absent agent, a held ticket — surfaces as a thrown ActionError carrying the
// chartr's specific message, whatever chartr's reason was.
//
// `force` is the operator's answer to the concurrency warning, sent only on the
// retry after they confirm it: it overrules the one-live-session-per-space gate and
// nothing else (ADR 0003 as amended). The first call always goes unforced, so the
// warning is the operator's to see rather than something the surface skips.
export function spawnSession(
  id: string,
  slug: string,
  num: number,
  role: string,
  agent = '',
  force = false,
): Promise<SpawnResult> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(id)}/maps/${encodeURIComponent(slug)}/tickets/${num}/spawn`,
    { role, agent, force },
  ) as Promise<SpawnResult>
}

// The death halt (ticket 10): a session whose process exited stays pinned to its
// ticket, and the operator resolves it exactly one of three ways — each a plain
// HTTP action, so chartr itself never acts. resumeSession relaunches the same
// session on its own ticket (same-ticket crash recovery, its claim standing);
// respawnSession starts a fresh session on the same ticket (a new claim supersedes
// the stale one); releaseSession clears the claim back to the frontier. The
// resulting state — a live tab again, or the ticket back on the frontier — arrives
// over the control socket; a refusal surfaces as a thrown ActionError.
//
// Resume and respawn seat a session, so both meet the same one-live-session gate a
// fresh spawn does and both take the same `force` the operator's confirmation
// sends. Release clears a claim and seats nothing, so it has no such gate.
export function resumeSession(spaceId: string, sessionId: string, force = false): Promise<unknown> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(spaceId)}/sessions/${encodeURIComponent(sessionId)}/resume`,
    { force },
  )
}

export function respawnSession(spaceId: string, sessionId: string, force = false): Promise<unknown> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(spaceId)}/sessions/${encodeURIComponent(sessionId)}/respawn`,
    { force },
  )
}

export function releaseSession(spaceId: string, sessionId: string): Promise<unknown> {
  return send('POST', `/api/spaces/${encodeURIComponent(spaceId)}/sessions/${encodeURIComponent(sessionId)}/release`)
}

// releaseTicket clears a claim addressed by the ticket that carries it rather than
// by the session holding it — the way back for a claim whose tab is gone: a chartr
// restart (tabs live in memory, the claim on disk does not), a dismissed dead tab,
// or a claim committed on another machine. Same write and same commit as
// releaseSession; refused with LIVE_SESSION while a live session still holds the
// ticket, which is the one case where the claim is not stale at all.
export function releaseTicket(spaceId: string, slug: string, num: number): Promise<unknown> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(spaceId)}/maps/${encodeURIComponent(slug)}/tickets/${num}/release`,
  )
}

// OpenLayerResult reports how far the open action got: `editor` launched
// $VISUAL/$EDITOR, `os` fell back to the OS opener, and `none` means the path
// itself is the answer — a layer with nothing on disk yet, or an environment with
// no way to open it.
export interface OpenLayerResult {
  path: string
  exists: boolean
  opened: 'editor' | 'os' | 'none'
  with?: string
}

// openConfigLayer opens a config file in the operator's editor. `layer` is a
// *name* the server resolves to a path (`user-config`, `terminal-config`, …); a
// local server never opens a path the client sent.
export function openConfigLayer(id: string, layer: string): Promise<OpenLayerResult> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/config/open`, {
    layer,
  }) as Promise<OpenLayerResult>
}

// openGlobalLayer is the same open for the layers that belong to no space — the
// operator's own config files. The settings route's
// global scope is reachable with nothing registered, so it never borrows a space
// id to open its own files.
export function openGlobalLayer(layer: string): Promise<OpenLayerResult> {
  return send('POST', '/api/config/open', { layer }) as Promise<OpenLayerResult>
}

// createConfigLayer stamps a config file from its defaults template — the
// companion to opening a layer that does not exist yet. `layer` is a *name* the
// server resolves and only creates when it carries a bundled template (today just
// `terminal-config`); an already-present file is refused rather than clobbered.
// The freshly-created file rides the next model snapshot over the control socket,
// so the row flips from "not created yet" to openable with no optimistic state.
export function createConfigLayer(layer: string): Promise<{ path: string; created: boolean }> {
  return send('POST', '/api/config/create', { layer }) as Promise<{
    path: string
    created: boolean
  }>
}

// setAgent registers or updates one agent of the operator's library. It is a PUT
// because the body is the agent's whole spec: what is sent is what the agent
// becomes, so a flag removed here is removed on disk rather than merged back in.
// Global, like the library itself — no space id, and it works with nothing
// registered at all.
export function setAgent(
  name: string,
  agent: { adapter: string; args?: string[]; env?: string[]; prompt?: string },
): Promise<{ name: string; path: string }> {
  return send('PUT', `/api/config/agents/${encodeURIComponent(name)}`, agent) as Promise<{
    name: string
    path: string
  }>
}

// deleteAgent removes one agent from the library. A space that last spawned with
// it simply reads nothing remembered on its next snapshot and reopens the picker —
// there is no assignment for a delete to strand.
export function deleteAgent(name: string): Promise<{ name: string; path: string }> {
  return send('DELETE', `/api/config/agents/${encodeURIComponent(name)}`) as Promise<{
    name: string
    path: string
  }>
}
