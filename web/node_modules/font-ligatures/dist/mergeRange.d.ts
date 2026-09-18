/**
 * Merges the range defined by the provided start and end into the list of
 * existing ranges. The merge is done in place on the existing range for
 * performance and is also returned.
 *
 * @param ranges Existing range list
 * @param newRangeStart Start position of the range to merge, inclusive
 * @param newRangeEnd End position of range to merge, exclusive
 */
export default function mergeRange(ranges: [number, number][], newRangeStart: number, newRangeEnd: number): [number, number][];
