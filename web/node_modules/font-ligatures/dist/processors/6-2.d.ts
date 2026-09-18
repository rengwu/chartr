import { ChainingContextualSubstitutionTable, Lookup } from '../tables';
import { LookupTree } from '../types';
/**
 * Build lookup tree for GSUB lookup table 6, format 2.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#62-chaining-context-substitution-format-2-class-based-glyph-contexts
 *
 * @param table JSON representation of the table
 * @param lookups List of lookup tables
 * @param tableIndex Index of this table in the overall lookup
 */
export default function buildTree(table: ChainingContextualSubstitutionTable.Format2, lookups: Lookup[], tableIndex: number): LookupTree;
