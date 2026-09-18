import { LookupTree } from './types';
/**
 * Merges the provided trees into a single lookup tree. When conflicting lookups
 * are encountered between two trees, the one with the lower index, then the
 * lower subindex is chosen.
 *
 * @param trees Array of trees to merge. Entries in earlier trees are favored
 *              over those in later trees when there is a choice.
 */
export default function mergeTrees(trees: LookupTree[]): LookupTree;
