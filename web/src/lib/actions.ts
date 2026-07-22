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
//
// It names its agent like every other spawn (ticket 03): ideate used to borrow
// the `grill` role's binding, which appeared on no surface. There is no role
// behind it to fall back on, so the name is always sent.
export function ideate(id: string, agent = ''): Promise<{ id: string }> {
  return send('POST', `/api/spaces/${encodeURIComponent(id)}/ideate`, {
    agent,
  }) as Promise<{
    id: string
  }>
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
export function spawnSession(
  id: string,
  slug: string,
  num: number,
  role: string,
  agent = '',
): Promise<SpawnResult> {
  return send(
    'POST',
    `/api/spaces/${encodeURIComponent(id)}/maps/${encodeURIComponent(slug)}/tickets/${num}/spawn`,
    { role, agent },
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
  field: 'adapter' | 'args' | 'prompt' | 'agent',
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

// openGlobalLayer is the same open for the layers that belong to no space — the
// operator's own config file and the global skill library. The settings route's
// global scope is reachable with nothing registered, so it never borrows a space
// id to open its own files.
export function openGlobalLayer(layer: string): Promise<OpenLayerResult> {
  return send('POST', '/api/config/open', { layer }) as Promise<OpenLayerResult>
}

// setAgent registers or updates one agent of the operator's library. It is a PUT
// because the body is the agent's whole spec: what is sent is what the agent
// becomes, so a flag removed here is removed on disk rather than merged back in.
// Global, like the library itself — no space id, and it works with nothing
// registered at all.
export function setAgent(
  name: string,
  agent: { adapter: string; args?: string[]; prompt?: string },
): Promise<{ name: string; path: string }> {
  return send('PUT', `/api/config/agents/${encodeURIComponent(name)}`, agent) as Promise<{
    name: string
    path: string
  }>
}

// deleteAgent removes one agent from the library. Roles assigned to it are left
// exactly as they are — the response names them, and each resolves to a visible
// warning with the role falling back to its own fields, rather than a delete here
// quietly rewriting a space's bindings.
export function deleteAgent(name: string): Promise<{ name: string; assigned?: string[] }> {
  return send('DELETE', `/api/config/agents/${encodeURIComponent(name)}`) as Promise<{
    name: string
    assigned?: string[]
  }>
}
