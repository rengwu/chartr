// The cockpit's routing, such as it is (ticket 05). There is exactly one route
// besides the cockpit itself — the global config surface — so this is a hash
// prefix and a parser, not a routing library.
//
// Two hash schemes share the bar and never collide, because one is prefixed with
// a slash and the other never is:
//
//   #/settings                     the settings route
//   #s=<spaceId>&m=<slug>&t=<num>  a star deep link
//
// The star link is untouched by this file; anything that does not start with
// `/settings` is the cockpit, and SpacePane goes on reading it exactly as before.

const prefix = '/settings'

export interface Route {
  /** True while the settings route is showing in place of the space cockpit. */
  settings: boolean
}

const cockpit: Route = { settings: false }

/** parseRoute reads a `location.hash` (with or without its leading `#`). */
export function parseRoute(hash: string): Route {
  const h = hash.startsWith('#') ? hash.slice(1) : hash
  // Anything under `/settings` is the one settings surface — a typo in the sub-path
  // should still land there rather than silently back on the cockpit.
  if (h !== prefix && !h.startsWith(prefix + '/')) return cockpit
  return { settings: true }
}

/** settingsHash is the hash for the settings surface — the inverse of parseRoute. */
export function settingsHash(): string {
  return '#' + prefix
}

/**
 * mapsHash is the deep link to a space's star-map picker — the grid of the
 * space's maps, and the door into any one of them.
 */
export function mapsHash(spaceId: string): string {
  return `#s=${encodeURIComponent(spaceId)}&maps=1`
}
