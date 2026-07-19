import type { Model } from './model'

export type ConnStatus = 'connecting' | 'open' | 'closed'

/**
 * ControlSocket owns the one JSON control socket per browser. It receives the
 * whole derived model as a snapshot on connect and again on every change, and
 * re-syncs on reconnect (ADR 0010). The client never writes state through it,
 * so a dropped connection loses nothing — the next snapshot is whole.
 *
 * `model` and `status` are runes, so any component reading them re-renders when
 * a snapshot lands or the connection changes.
 */
export class ControlSocket {
  model = $state<Model | null>(null)
  status = $state<ConnStatus>('connecting')

  #url: string
  #ws: WebSocket | null = null
  #reconnectDelay = 500
  #closed = false

  constructor(url: string = defaultUrl()) {
    this.#url = url
  }

  connect(): void {
    if (this.#closed) return
    this.status = this.status === 'open' ? this.status : 'connecting'

    const ws = new WebSocket(this.#url)
    this.#ws = ws

    ws.onopen = () => {
      this.status = 'open'
      this.#reconnectDelay = 500
    }
    ws.onmessage = (ev: MessageEvent<string>) => {
      this.model = JSON.parse(ev.data) as Model
    }
    ws.onerror = () => ws.close()
    ws.onclose = () => {
      if (this.#closed) return
      this.status = 'closed'
      setTimeout(() => this.connect(), this.#reconnectDelay)
      // Back off to a ceiling so a downed backend does not spin the browser.
      this.#reconnectDelay = Math.min(this.#reconnectDelay * 2, 10_000)
    }
  }

  /** Stop reconnecting and close the socket. */
  close(): void {
    this.#closed = true
    this.#ws?.close()
  }
}

function defaultUrl(): string {
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${proto}//${location.host}/ws/control`
}
