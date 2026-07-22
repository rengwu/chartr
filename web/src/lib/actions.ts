// Operator actions are plain HTTP request/response (ADR 0010): a failed action
// surfaces as a response the caller shows, never as a silent state change. The
// resulting state arrives separately over the control socket as a fresh whole
// snapshot, so these helpers return only the action's own result (or throw).

export class ActionError extends Error {}

async function send(method: string, path: string, body?: unknown): Promise<unknown> {
  const res = await fetch(path, {
    method,
    headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  })
  if (!res.ok) {
    let msg = `${method} ${path} failed (${res.status})`
    try {
      const data = (await res.json()) as { error?: string }
      if (data?.error) msg = data.error
    } catch {
      // Non-JSON error body; keep the status-line message.
    }
    throw new ActionError(msg)
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

export function deregisterSpace(id: string): Promise<void> {
  return send('DELETE', `/api/spaces/${encodeURIComponent(id)}`) as Promise<void>
}

export function setPin(id: string, pinned: boolean): Promise<void> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/pin`, { pinned }) as Promise<void>
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

// ideate opens the ideate on-ramp (ticket 15): a live, ticketless agent tab typed
// the on-disk starter prompt on open. It shares only the adapter's spawn
// primitive with a real session — no map or ticket, no claim, no lifecycle, ended
// only by the human, exactly like an ad-hoc shell — and returns the new tab's id.
export function ideate(id: string): Promise<{ id: string }> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/ideate`) as Promise<{
    id: string
  }>
}

// previewPayload composes what a session for one ticket and role would be told
// (ticket 08) — the resolved skills and the context bundle — with per-part
// provenance. Read-only inspection, so a GET; the
// harness reads the library and the map fresh, so an edit on disk shows up here.
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

// SpawnResult is the spawn action's own response — the session it started, the
// resolved agent and model, and the payload hash the claim commit recorded. The
// live session tab arrives separately over the control socket.
export interface SpawnResult {
  sessionId: string
  ticketNum: number
  role: string
  agent: string
  model: string
  payloadSha: string
}

// spawnSession spawns a session on a frontier ticket (ticket 09): the harness
// writes the claim commit, composes and archives the payload, and launches the
// bound agent's TUI with the read-this-file opener typed in. A blocked spawn — an
// absent agent, an unclassified map, a held ticket — surfaces as a thrown
// ActionError carrying the harness's specific message.
export function spawnSession(
  id: string,
  slug: string,
  num: number,
  role: string,
): Promise<SpawnResult> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(id)}/maps/${encodeURIComponent(slug)}/tickets/${num}/spawn`,
    { role },
  ) as Promise<SpawnResult>
}

// The death halt (ticket 10): a session whose process exited stays pinned to its
// ticket, and the operator resolves it exactly one of three ways — each a plain
// HTTP action, so the harness itself never acts. resumeSession relaunches the same
// session on its own ticket (same-ticket crash recovery, its claim standing);
// respawnSession starts a fresh session on the same ticket (a new claim supersedes
// the stale one); releaseSession clears the claim back to the frontier. The
// resulting state — a live tab again, or the ticket back on the frontier — arrives
// over the control socket; a refusal surfaces as a thrown ActionError.
export function resumeSession(spaceId: string, sessionId: string): Promise<unknown> {
  return send('POST', `/api/spaces/${encodeURIComponent(spaceId)}/sessions/${encodeURIComponent(sessionId)}/resume`)
}

export function respawnSession(spaceId: string, sessionId: string): Promise<unknown> {
  return send('POST', `/api/spaces/${encodeURIComponent(spaceId)}/sessions/${encodeURIComponent(sessionId)}/respawn`)
}

export function releaseSession(spaceId: string, sessionId: string): Promise<unknown> {
  return send('POST', `/api/spaces/${encodeURIComponent(spaceId)}/sessions/${encodeURIComponent(sessionId)}/release`)
}

// setBinding edits one field of one role binding from the transparency surface
// (ticket 05). It writes the **user layer and nowhere else** — bindings resolve
// user-over-workspace (ADR 0009), so that is their home, and the cockpit never
// writes a space's committed config. Pass null to clear the override, which
// reveals the layer beneath it; `args` takes a list. The edited value and its new
// provenance arrive over the control socket, so there is no optimistic state here.
export function setBinding(
  id: string,
  role: string,
  field: 'adapter' | 'model' | 'args',
  value: string | string[] | null,
): Promise<{ role: string; field: string; cleared: boolean; path: string }> {
  return send('PUT', `/api/spaces/${encodeURIComponent(id)}/config/binding`, {
    role,
    field,
    value,
  }) as Promise<{ role: string; field: string; cleared: boolean; path: string }>
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

// openConfigLayer opens a config layer in the operator's editor — the escape
// hatch for everything the surface deliberately does not edit inline. `layer` is
// a *name* the server resolves to a path (`workspace-config`, `user-config`,
// `skill:<name>`, …); a local server never opens a path the client sent.
export function openConfigLayer(id: string, layer: string): Promise<OpenLayerResult> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/config/open`, {
    layer,
  }) as Promise<OpenLayerResult>
}

// classifyMap declares a map's kind (ADR 0007), writing it into the space's
// committed workspace config. The new classification arrives over the control
// socket like any other state; this returns only the action's own result.
export function classifyMap(
  id: string,
  slug: string,
  kind: 'planning' | 'implementation',
): Promise<{ slug: string; kind: string }> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(id)}/maps/${encodeURIComponent(slug)}/classify`,
    { kind },
  ) as Promise<{ slug: string; kind: string }>
}
