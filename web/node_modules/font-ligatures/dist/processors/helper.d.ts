import { LookupTreeEntry, LookupTree } from '../types';
import { SubstitutionLookupRecord, Lookup } from '../tables';
export interface EntryMeta {
    entry: LookupTreeEntry;
    substitutions: (number | null)[];
}
export declare function processInputPosition(glyphs: (number | [number, number])[], position: number, currentEntries: EntryMeta[], lookupRecords: SubstitutionLookupRecord[], lookups: Lookup[]): EntryMeta[];
export declare function processLookaheadPosition(glyphs: (number | [number, number])[], currentEntries: EntryMeta[]): EntryMeta[];
export declare function processBacktrackPosition(glyphs: (number | [number, number])[], currentEntries: EntryMeta[]): EntryMeta[];
export declare function getInputTree(tree: LookupTree, substitutions: SubstitutionLookupRecord[], lookups: Lookup[], inputIndex: number, glyphId: number | [number, number]): {
    entry: LookupTreeEntry;
    substitution: number | null;
}[];
