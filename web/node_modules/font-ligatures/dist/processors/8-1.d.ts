import { ReverseChainingContextualSingleSubstitutionTable } from '../tables';
import { LookupTree } from '../types';
/**
 * Build lookup tree for GSUB lookup table 8, format 1.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#81-reverse-chaining-contextual-single-substitution-format-1-coverage-based-glyph-contexts
 *
 * @param table JSON representation of the table
 * @param tableIndex Index of this table in the overall lookup
 */
export default function buildTree(table: ReverseChainingContextualSingleSubstitutionTable, tableIndex: number): LookupTree;
