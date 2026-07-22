// The cockpit's routing, such as it is (ticket 05). There is exactly one route
// besides the cockpit itself — the effective config surface — so this is a hash
// prefix and a parser, not a routing library.
//
// Two hash schemes share the bar and never collide, because one is prefixed with
// a slash and the other never is:
//
//   #/settings, #/settings/user, #/settings/s=<spaceId>   the settings route
//   #s=<spaceId>&m=<slug>&t=<num>                         a star deep link
//
// The star link is untouched by this file; anything that does not start with
// `/settings` is the cockpit, and SpacePane goes on reading it exactly as before.

const prefix = '/settings'

/** Which scope the settings route is showing. */
export type SettingsScope =
  /** One space's effective config. */
  | { kind: 'space'; spaceId: string }
  /** The one global user file, which is not a space's. */
  | { kind: 'user' }
  /** `#/settings` with no sub-path: the screen falls back to the open space. */
  | { kind: 'default' }

export interface Route {
  /** True while the settings route is showing in place of the space cockpit. */
  settings: boolean
  /** The scope the settings route is on; null when this is not the settings route. */
  scope: SettingsScope | null
}

const cockpit: Route = { settings: false, scope: null }

/** parseRoute reads a `location.hash` (with or without its leading `#`). */
export function parseRoute(hash: string): Route {
  const h = hash.startsWith('#') ? hash.slice(1) : hash
  if (h !== prefix && !h.startsWith(prefix + '/')) return cockpit

  const rest = h.slice(prefix.length).replace(/^\//, '')
  if (rest === '') return { settings: true, scope: { kind: 'default' } }
  if (rest === 'user') return { settings: true, scope: { kind: 'user' } }
  if (rest.startsWith('s=')) {
    const spaceId = decodeURIComponent(rest.slice(2))
    // An `s=` with nothing after it names no space; fall back rather than route
    // to a space that cannot exist.
    if (spaceId === '') return { settings: true, scope: { kind: 'default' } }
    return { settings: true, scope: { kind: 'space', spaceId } }
  }
  // An unknown sub-path is still the settings route — a typo in the bar should
  // land somewhere legible, not silently back on the cockpit.
  return { settings: true, scope: { kind: 'default' } }
}

/** settingsHash builds the hash for one scope — the inverse of parseRoute. */
export function settingsHash(scope: SettingsScope): string {
  switch (scope.kind) {
    case 'user':
      return '#' + prefix + '/user'
    case 'space':
      return '#' + prefix + '/s=' + encodeURIComponent(scope.spaceId)
    default:
      return '#' + prefix
  }
}

/**
 * mapsHash is the deep link to a space's star-map picker — where a map's kind is
 * declared (ADR 0007). The settings surface renders kinds read-only and links
 * here rather than growing a second way to classify.
 */
export function mapsHash(spaceId: string): string {
  return `#s=${encodeURIComponent(spaceId)}&maps=1`
}
