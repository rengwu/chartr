import { ChainingContextualSubstitutionTable, Lookup } from '../tables';
import { LookupTree } from '../types';
/**
 * Build lookup tree for GSUB lookup table 6, format 1.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#61-chaining-context-substitution-format-1-simple-glyph-contexts
 *
 * @param table JSON representation of the table
 * @param lookups List of lookup tables
 * @param tableIndex Index of this table in the overall lookup
 */
export default function buildTree(table: ChainingContextualSubstitutionTable.Format1, lookups: Lookup[], tableIndex: number): LookupTree;
