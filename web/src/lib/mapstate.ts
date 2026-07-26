// Where the star-map pane's state lives between the lives of its components.
//
// The cockpit renders one SpacePane and one star-map card for whichever space is
// selected, so switching space does not put a second pane beside the first — it
// re-points the one pane at other data, and the island underneath is torn down
// and rebuilt. Without somewhere outside that lifetime to keep it, everything the
// operator had set up — the open map, the star they were reading, where they had
// pushed the camera — dies on the switch and the map greets them fresh.
//
// So this module remembers it *per space*, and the pane hands its state over on
// the way out and asks for it back on the way in.
//
// Two horizons, deliberately different:
//
//   • across a space switch — everything: the open map, the selection, and each
//     map's camera pose. The session is still going; going next door and coming
//     back should return you to your desk exactly as you left it.
//   • across a reload or a restart — the open map, and nothing else. Which map
//     you work in is a standing fact about a space, worth carrying between
//     sittings. A camera pose and a half-read ticket are not: they belong to the
//     sitting that made them, and restoring them into a map whose stars have
//     moved since would be a worse greeting than the fit.
//
// Only the first horizon touches storage, which is why the camera is held in a
// plain Map and never serialized.

import type { Map as WMap } from './model'
import type { Camera } from './starmap/starmap'

/** One space's star-map pane, as the pane itself sees it. */
export interface MapPaneState {
  /** The open map's slug, or null for the picker. */
  slug: string | null
  /** The selected ticket number, or null. */
  selected: number | null
  /** Whether the pane is showing the map material rather than a ticket. */
  showMaterial: boolean
}

const EMPTY: MapPaneState = { slug: null, selected: null, showMaterial: false }

const panes = new Map<string, MapPaneState>()
const cameras = new Map<string, Camera>()

// The one persisted fact, as `{ [spaceId]: slug }`. Versioned in the key so a
// later shape change cannot be handed a payload it would have to guess at.
const STORE_KEY = 'chartr.mappane.v1'

/** What the pane last had open in this space — EMPTY for a space never opened. */
export function paneState(spaceId: string): MapPaneState {
  return { ...(panes.get(spaceId) ?? EMPTY) }
}

/** Record this space's pane state, persisting the open map for the next run. */
export function rememberPane(spaceId: string, state: MapPaneState): void {
  const prev = panes.get(spaceId)
  panes.set(spaceId, { ...state })
  if (!prev || prev.slug !== state.slug) persistSlugs()
}

/**
 * What the pane should open on when it swings to a space: the state remembered
 * for it, checked against the maps that space actually has now.
 *
 * The check matters because the two are written at different times. Maps are
 * charted, re-charted and deleted by agents in a shell while the cockpit watches;
 * a slug remembered before a restart, or a ticket number from ten minutes ago,
 * can each name something that has since stopped existing. So a remembered map
 * only re-opens if it is still there — otherwise the space is greeted the way a
 * fresh summon greets it, with its one map or the picker — and a remembered star
 * only comes back with its own map, and only while that ticket still exists.
 */
export function openingFor(state: MapPaneState, maps: WMap[]): MapPaneState {
  const known = !!state.slug && maps.some((m) => m.slug === state.slug)
  const slug = known ? state.slug : maps.length === 1 ? (maps[0]?.slug ?? null) : null
  if (!known) return { slug, selected: null, showMaterial: false }
  const open = maps.find((m) => m.slug === slug)
  const alive = state.selected !== null && !!open?.tickets.some((t) => t.num === state.selected)
  return {
    slug,
    selected: alive ? state.selected : null,
    // The material and a star are one pane showing one thing; a star that has
    // gone does not fall back to the material it was never showing.
    showMaterial: state.selected === null && state.showMaterial,
  }
}

/**
 * The key one map's camera is remembered under. Space *and* slug: two spaces can
 * hold maps of the same name, and a camera pose belongs to one constellation.
 */
export function cameraKey(spaceId: string, slug: string): string {
  return `${spaceId} ${slug}`
}

/** The pose this map was last left at, or null — the island then fits instead. */
export function cameraOf(key: string): Camera | null {
  const c = cameras.get(key)
  return c ? { ...c } : null
}

export function rememberCamera(key: string, cam: Camera): void {
  cameras.set(key, { ...cam })
}

/** Drop everything held for a space — it has been forgotten by the cockpit. */
export function forgetSpace(spaceId: string): void {
  panes.delete(spaceId)
  for (const key of cameras.keys()) {
    if (key.startsWith(spaceId + ' ')) cameras.delete(key)
  }
  persistSlugs()
}

function persistSlugs(): void {
  const slugs: Record<string, string> = {}
  for (const [id, st] of panes) if (st.slug) slugs[id] = st.slug
  try {
    localStorage.setItem(STORE_KEY, JSON.stringify(slugs))
  } catch {
    // Storage disabled or full: the pane keeps working for this sitting, it just
    // won't greet the next one with the same map open.
  }
}

// Seed the open maps from the last run. A stale slug — a map deleted between
// runs — is not a problem to solve here: the card falls back to the picker when
// the slug names no map in the space, which is the same path a bad deep link
// takes.
function load(): void {
  let raw: string | null = null
  try {
    raw = localStorage.getItem(STORE_KEY)
  } catch {
    return
  }
  if (!raw) return
  try {
    const slugs = JSON.parse(raw) as unknown
    if (!slugs || typeof slugs !== 'object') return
    for (const [id, slug] of Object.entries(slugs as Record<string, unknown>)) {
      if (typeof slug === 'string' && slug) panes.set(id, { ...EMPTY, slug })
    }
  } catch {
    // Corrupt payload; start clean rather than fight it.
  }
}

load()
