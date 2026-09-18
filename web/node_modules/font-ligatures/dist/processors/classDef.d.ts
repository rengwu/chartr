import { ClassDefTable } from '../tables';
/**
 * Get the number of the class to which the glyph belongs, or null if it doesn't
 * belong to any of them.
 *
 * @param table JSON representation of the class def table
 * @param glyphId Index of the glyph to look for
 */
export default function getGlyphClass(table: ClassDefTable, glyphId: number | [number, number]): Map<number | [number, number], number | null>;
export declare function listClassGlyphs(table: ClassDefTable, index: number): (number | [number, number])[];
