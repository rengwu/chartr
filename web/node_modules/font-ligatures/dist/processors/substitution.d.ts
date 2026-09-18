import { SubstitutionTable } from '../tables';
/**
 * Get the substitution glyph for the givne glyph, or null if the glyph was not
 * found in the table.
 *
 * @param table JSON representation of the substitution table
 * @param glyphId The index of the glpyh to find substitutions for
 */
export declare function getRangeSubstitutionGlyphs(table: SubstitutionTable, glyphId: [number, number]): Map<[number, number] | number, number | null>;
export declare function getIndividualSubstitutionGlyph(table: SubstitutionTable, glyphId: number): number | null;
