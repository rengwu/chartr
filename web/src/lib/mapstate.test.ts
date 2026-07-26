// The star-map pane's memory has two horizons and the difference between them is
// the whole point: a space switch returns you to your desk exactly as you left
// it, a restart returns you only to the map you work in. These pin both, and the
// keying that keeps two spaces' maps of the same name apart.

import { describe, it, expect, beforeEach, vi } from 'vitest'
import type { Map as WMap } from './model'

const STORE_KEY = 'chartr.mappane.v1'

// The module seeds itself from storage at import, so each test that cares about
// a cold start has to import it fresh against a prepared store.
async function load() {
  vi.resetModules()
  return import('./mapstate')
}

beforeEach(() => {
  localStorage.clear()
})

describe('across a space switch', () => {
  it('hands back the map, the star and the pane it was given', async () => {
    const { paneState, rememberPane } = await load()
    rememberPane('space-1', { slug: 'delivery', selected: 7, showMaterial: false })
    expect(paneState('space-1')).toEqual({
      slug: 'delivery',
      selected: 7,
      showMaterial: false,
    })
  })

  it('keeps every space to its own state', async () => {
    const { paneState, rememberPane } = await load()
    rememberPane('space-1', { slug: 'delivery', selected: 7, showMaterial: false })
    rememberPane('space-2', { slug: 'onboarding', selected: null, showMaterial: true })
    expect(paneState('space-1').slug).toBe('delivery')
    expect(paneState('space-2')).toEqual({
      slug: 'onboarding',
      selected: null,
      showMaterial: true,
    })
  })

  it('gives a space never opened an empty pane, not its neighbour', async () => {
    const { paneState, rememberPane } = await load()
    rememberPane('space-1', { slug: 'delivery', selected: 7, showMaterial: false })
    expect(paneState('space-2')).toEqual({ slug: null, selected: null, showMaterial: false })
  })

  it('hands out copies, so a caller mutating what it got cannot rewrite the store', async () => {
    const { paneState, rememberPane } = await load()
    const written = { slug: 'delivery', selected: 7, showMaterial: false }
    rememberPane('space-1', written)
    written.slug = 'elsewhere'
    const read = paneState('space-1')
    read.selected = 99
    expect(paneState('space-1')).toEqual({
      slug: 'delivery',
      selected: 7,
      showMaterial: false,
    })
  })

  it('remembers a camera pose per space and map', async () => {
    const { cameraKey, cameraOf, rememberCamera } = await load()
    rememberCamera(cameraKey('space-1', 'delivery'), { x: -120, y: 40, s: 1.7 })
    rememberCamera(cameraKey('space-2', 'delivery'), { x: 10, y: 10, s: 0.4 })
    expect(cameraOf(cameraKey('space-1', 'delivery'))).toEqual({ x: -120, y: 40, s: 1.7 })
    // Same map name, other space: a pose belongs to one constellation.
    expect(cameraOf(cameraKey('space-2', 'delivery'))).toEqual({ x: 10, y: 10, s: 0.4 })
    expect(cameraOf(cameraKey('space-1', 'onboarding'))).toBeNull()
  })
})

describe('across a restart', () => {
  it('carries the open map over, and nothing else', async () => {
    const first = await load()
    first.rememberPane('space-1', { slug: 'delivery', selected: 7, showMaterial: true })
    first.rememberCamera(first.cameraKey('space-1', 'delivery'), { x: -120, y: 40, s: 1.7 })

    // A fresh load of the module is a fresh run of the app.
    const next = await load()
    expect(next.paneState('space-1')).toEqual({
      slug: 'delivery',
      selected: null,
      showMaterial: false,
    })
    expect(next.cameraOf(next.cameraKey('space-1', 'delivery'))).toBeNull()
  })

  it('carries nothing over for a space left on the picker', async () => {
    const first = await load()
    first.rememberPane('space-1', { slug: null, selected: null, showMaterial: false })
    const next = await load()
    expect(next.paneState('space-1').slug).toBeNull()
  })

  it('starts clean on a corrupt or foreign payload rather than throwing', async () => {
    localStorage.setItem(STORE_KEY, '{not json')
    let mod = await load()
    expect(mod.paneState('space-1').slug).toBeNull()

    localStorage.setItem(STORE_KEY, JSON.stringify({ 'space-1': 42 }))
    mod = await load()
    expect(mod.paneState('space-1').slug).toBeNull()
  })
})

// What the pane opens on when it swings to a space, checked against the maps
// that space has *now*: agents chart, re-chart and delete maps in a shell while
// the cockpit watches, so anything remembered can name something already gone.
describe('the opening state for a space', () => {
  function map(slug: string, tickets: number[]): WMap {
    return {
      slug,
      name: slug,
      dir: `.plan/maps/${slug}`,
      destination: '',
      finished: false,
      tickets: tickets.map((num) => ({
        num,
        slug: `${num}`,
        title: `#${num}`,
        type: 'task',
        status: 'open',
        frontier: false,
        blockedBy: [],
      })),
    }
  }
  const maps = [map('delivery', [1, 2, 3]), map('onboarding', [1])]

  it('re-opens the remembered map, star and all', async () => {
    const { openingFor } = await load()
    expect(openingFor({ slug: 'delivery', selected: 2, showMaterial: false }, maps)).toEqual({
      slug: 'delivery',
      selected: 2,
      showMaterial: false,
    })
  })

  it('re-opens the remembered material', async () => {
    const { openingFor } = await load()
    expect(openingFor({ slug: 'delivery', selected: null, showMaterial: true }, maps)).toEqual({
      slug: 'delivery',
      selected: null,
      showMaterial: true,
    })
  })

  it('drops a star whose ticket has left the map since', async () => {
    const { openingFor } = await load()
    expect(openingFor({ slug: 'onboarding', selected: 3, showMaterial: false }, maps)).toEqual({
      slug: 'onboarding',
      selected: null,
      showMaterial: false,
    })
  })

  it('falls to the picker when the remembered map is gone', async () => {
    const { openingFor } = await load()
    expect(openingFor({ slug: 'deleted', selected: 2, showMaterial: true }, maps)).toEqual({
      slug: null,
      selected: null,
      showMaterial: false,
    })
  })

  it('greets a space it has nothing for the way a fresh summon does', async () => {
    const { openingFor } = await load()
    const empty = { slug: null, selected: null, showMaterial: false }
    // One map is no choice at all — open it, exactly as summoning would.
    expect(openingFor(empty, [map('only', [1])]).slug).toBe('only')
    // Several is a choice: the picker.
    expect(openingFor(empty, maps).slug).toBeNull()
    // None at all is the empty picker.
    expect(openingFor(empty, []).slug).toBeNull()
  })
})

describe('forgetting a space', () => {
  it('drops its pane and its cameras, from memory and from storage', async () => {
    const { paneState, rememberPane, rememberCamera, cameraOf, cameraKey, forgetSpace } =
      await load()
    rememberPane('space-1', { slug: 'delivery', selected: 7, showMaterial: false })
    rememberPane('space-2', { slug: 'onboarding', selected: null, showMaterial: false })
    rememberCamera(cameraKey('space-1', 'delivery'), { x: 1, y: 2, s: 1 })
    rememberCamera(cameraKey('space-2', 'onboarding'), { x: 3, y: 4, s: 1 })

    forgetSpace('space-1')

    expect(paneState('space-1').slug).toBeNull()
    expect(cameraOf(cameraKey('space-1', 'delivery'))).toBeNull()
    // The space beside it is untouched, in memory and on disk.
    expect(paneState('space-2').slug).toBe('onboarding')
    expect(cameraOf(cameraKey('space-2', 'onboarding'))).toEqual({ x: 3, y: 4, s: 1 })
    expect(JSON.parse(localStorage.getItem(STORE_KEY)!)).toEqual({ 'space-2': 'onboarding' })
  })
})
