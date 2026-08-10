import type { Space } from './model'

/**
 * The one visibility rule for the cockpit: registered spaces are always shown;
 * Scratch is shown only while it holds an open shell. The server keeps Scratch
 * in every snapshot so ordering remains authoritative there.
 */
export function visibleSpaces(spaces: Space[]): Space[] {
  return spaces.filter((space) => !space.scratch || space.terminals.length > 0)
}
