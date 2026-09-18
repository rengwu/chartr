import { ChainingContextualSubstitutionTable, Lookup } from '../tables';
import { LookupTree } from '../types';
/**
 * Build lookup tree for GSUB lookup table 6, format 3.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#63-chaining-context-substitution-format-3-coverage-based-glyph-contexts
 *
 * @param table JSON representation of the table
 * @param lookups List of lookup tables
 * @param tableIndex Index of this table in the overall lookup
 */
export default function buildTree(table: ChainingContextualSubstitutionTable.Format3, lookups: Lookup[], tableIndex: number): LookupTree;
