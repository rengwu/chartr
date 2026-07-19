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
