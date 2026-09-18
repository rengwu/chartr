"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Get the index of the given glyph in the coverage table, or null if it is not
 * present in the table.
 *
 * @param table JSON representation of the coverage table
 * @param glyphId Index of the glyph to look for
 */
function getCoverageGlyphIndex(table, glyphId) {
    switch (table.format) {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-format-1
        case 1:
            const index = table.glyphs.indexOf(glyphId);
            return index !== -1
                ? index
                : null;
        // https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-format-2
        case 2:
            const range = table.ranges
                .find(range => range.start <= glyphId && range.end >= glyphId);
            return range
                ? range.index
                : null;
    }
}
exports.default = getCoverageGlyphIndex;
function listGlyphsByIndex(table) {
    switch (table.format) {
        case 1:
            return table.glyphs.map((glyphId, index) => ({ glyphId, index }));
        case 2:
            let results = [];
            for (const [index, range] of table.ranges.entries()) {
                if (range.end === range.start) {
                    results.push({ glyphId: range.start, index });
                }
                else {
                    results.push({ glyphId: [range.start, range.end + 1], index });
                }
            }
            return results;
    }
}
exports.listGlyphsByIndex = listGlyphsByIndex;
//# sourceMappingURL=coverage.js.map