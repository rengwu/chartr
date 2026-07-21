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

// previewPayload composes what a session for one ticket and role would be told
// (ticket 08) — the resolved prompts, the context bundle, and the review
// guarantees — with per-part provenance. Read-only inspection, so a GET; the
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

// The human review hub (ticket 12). The hub renders the brief the harness wrote
// to disk and adds buttons — these are those buttons, each a plain HTTP action so
// a refusal (an unacknowledged blocking finding, a live session, a ticket that has
// moved on) surfaces as a thrown ActionError carrying the harness's own message.

export interface ReviewFinding {
  text: string
  // The Done-when clause the finding cites. Empty means it cites none, which is
  // what makes it advisory by rule — the structural way a nitpick is told from a
  // real bug.
  clause: string
}

// ReviewRead is the brief a human reads at the gate, plus the mechanical shape
// the hub's buttons key on. `brief` is the exact markdown on disk — the GUI adds
// buttons and nothing else, so a CLI-only operator reads the same text.
export interface ReviewRead {
  sessionId: string
  ticketNum: number
  brief: string
  recommendation: 'Approve' | 'Send back'
  verdictLine: string
  blocking: ReviewFinding[]
  advisories: ReviewFinding[]
  proposedAnswer: string
}

export function readReview(id: string, slug: string, num: number): Promise<ReviewRead> {
  return send('GET', ticketPath(id, slug, num) + '/review') as Promise<ReviewRead>
}

// ApproveResult is what the approval bought: the promotion commit, the dependents
// it unblocked, and the next best frontier ticket the post-approve strip offers.
// `smearedInto` is ADR 0008's residual race, reported rather than hidden — the
// answer is promoted, but another writer's commit carries the edit.
export interface ApproveResult {
  ticketNum: number
  commit: string
  unblocked?: number[]
  approvedOverRejection: boolean
  next?: { num: number; title: string }
  smearedInto?: string
  warning?: string
}

// approveTicket promotes the `## Proposed Answer` to `## Answer` as its own
// pathspec-limited commit. Over a rejecting verdict it costs exactly one tick,
// which the harness refuses the approval without.
export function approveTicket(
  id: string,
  slug: string,
  num: number,
  acknowledged: boolean,
): Promise<ApproveResult> {
  return send('POST', ticketPath(id, slug, num) + '/approve', {
    acknowledged,
  }) as Promise<ApproveResult>
}

// followUp stacks another session on a still-proposed ticket — the mechanism
// behind both "send back to fix" (with the blocking finding attached and any
// advisories the operator ticked) and "take it further". The note and the findings
// ride the injected payload and its archive, never the ticket file.
export function followUp(
  id: string,
  slug: string,
  num: number,
  body: { role?: string; note?: string; advisories?: number[]; includeFindings?: boolean },
): Promise<SpawnResult & { followUp: boolean }> {
  return send('POST', ticketPath(id, slug, num) + '/follow-up', body) as Promise<
    SpawnResult & { followUp: boolean }
  >
}

export interface AbandonResult {
  ticketNum: number
  commit: string
  workCommits?: string[]
  reverted: boolean
  reset: boolean
  revertError?: string
}

// abandonTicket rejects the proposal, not the ticket: the reason is demoted into
// the ticket as dated `### Rejected` prose and the ticket returns to the frontier.
// It destroys nothing unless a lever is ticked.
export function abandonTicket(
  id: string,
  slug: string,
  num: number,
  body: { reason: string; revert?: boolean; reset?: boolean },
): Promise<AbandonResult> {
  return send('POST', ticketPath(id, slug, num) + '/abandon', body) as Promise<AbandonResult>
}

export type DiffScope = 'all' | 'verdict' | 'read'

export interface TicketDiff {
  scope: DiffScope
  base: string
  head: string
  patch: string
  stat: string
  note?: string
}

// ticketDiff serves the work under a proposal at one of three scopes: all the
// commits since the ticket was claimed, everything since the verdict being read,
// or everything since the operator's last read (they pass the sha they last saw).
export function ticketDiff(
  id: string,
  slug: string,
  num: number,
  scope: DiffScope,
  since?: string,
): Promise<TicketDiff> {
  const q = new URLSearchParams({ scope })
  if (since) q.set('since', since)
  return send('GET', ticketPath(id, slug, num) + '/diff?' + q.toString()) as Promise<TicketDiff>
}

function ticketPath(id: string, slug: string, num: number): string {
  return `/api/spaces/${encodeURIComponent(id)}/maps/${encodeURIComponent(slug)}/tickets/${num}`
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
