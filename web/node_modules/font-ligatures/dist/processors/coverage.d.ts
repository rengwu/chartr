import { CoverageTable } from '../tables';
/**
 * Get the index of the given glyph in the coverage table, or null if it is not
 * present in the table.
 *
 * @param table JSON representation of the coverage table
 * @param glyphId Index of the glyph to look for
 */
export default function getCoverageGlyphIndex(table: CoverageTable, glyphId: number): number | null;
export declare function listGlyphsByIndex(table: CoverageTable): {
    glyphId: number | [number, number];
    index: number;
}[];
