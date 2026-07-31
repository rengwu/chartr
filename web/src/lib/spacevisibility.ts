import type { Space } from './model'

/**
 * The one visibility rule for the cockpit: registered spaces are always shown;
 * Scratch is shown only while it holds an open shell. The server keeps Scratch
 * in every snapshot so ordering remains authoritative there.
 */
export function visibleSpaces(spaces: Space[]): Space[] {
  return spaces.filter((space) => !space.scratch || space.terminals.length > 0)
}

/**
 * The spaces the settings surface enumerates as config scopes. Scratch has no
 * config to scope — no committed layers, no repository to hold them — so it is
 * never one.
 *
 * It reads the *unfiltered* snapshot rather than `visibleSpaces` above, because
 * the two rules answer different questions: the sidebar hides an empty Scratch,
 * but a Scratch space that happens to have a shell open must not appear here
 * either.
 */
export function configurableSpaces(spaces: Space[]): Space[] {
  return spaces.filter((space) => !space.scratch)
}
