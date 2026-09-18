"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const coverage_1 = require("./coverage");
const helper_1 = require("./helper");
/**
 * Build lookup tree for GSUB lookup table 8, format 1.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#81-reverse-chaining-contextual-single-substitution-format-1-coverage-based-glyph-contexts
 *
 * @param table JSON representation of the table
 * @param tableIndex Index of this table in the overall lookup
 */
function buildTree(table, tableIndex) {
    const result = {
        individual: {},
        range: []
    };
    const glyphs = coverage_1.listGlyphsByIndex(table.coverage);
    for (const { glyphId, index } of glyphs) {
        const initialEntry = {};
        if (Array.isArray(glyphId)) {
            result.range.push({
                entry: initialEntry,
                range: glyphId
            });
        }
        else {
            result.individual[glyphId] = initialEntry;
        }
        let currentEntries = [{
                entry: initialEntry,
                substitutions: [table.substitutes[index]]
            }];
        // We walk forward, then backward
        for (const coverage of table.lookaheadCoverage) {
            currentEntries = helper_1.processLookaheadPosition(coverage_1.listGlyphsByIndex(coverage).map(glyph => glyph.glyphId), currentEntries);
        }
        for (const coverage of table.backtrackCoverage) {
            currentEntries = helper_1.processBacktrackPosition(coverage_1.listGlyphsByIndex(coverage).map(glyph => glyph.glyphId), currentEntries);
        }
        // When we get to the end, insert the lookup information
        for (const { entry, substitutions } of currentEntries) {
            entry.lookup = {
                substitutions,
                index: tableIndex,
                subIndex: 0,
                length: 1,
                contextRange: [
                    -1 * table.backtrackCoverage.length,
                    1 + table.lookaheadCoverage.length
                ]
            };
        }
    }
    return result;
}
exports.default = buildTree;
//# sourceMappingURL=8-1.js.map