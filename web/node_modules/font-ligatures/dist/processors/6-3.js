"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const coverage_1 = require("./coverage");
const helper_1 = require("./helper");
/**
 * Build lookup tree for GSUB lookup table 6, format 3.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#63-chaining-context-substitution-format-3-coverage-based-glyph-contexts
 *
 * @param table JSON representation of the table
 * @param lookups List of lookup tables
 * @param tableIndex Index of this table in the overall lookup
 */
function buildTree(table, lookups, tableIndex) {
    const result = {
        individual: {},
        range: []
    };
    const firstGlyphs = coverage_1.listGlyphsByIndex(table.inputCoverage[0]);
    for (const { glyphId } of firstGlyphs) {
        let currentEntries = helper_1.getInputTree(result, table.lookupRecords, lookups, 0, glyphId).map(({ entry, substitution }) => ({ entry, substitutions: [substitution] }));
        for (const [index, coverage] of table.inputCoverage.slice(1).entries()) {
            currentEntries = helper_1.processInputPosition(coverage_1.listGlyphsByIndex(coverage).map(glyph => glyph.glyphId), index + 1, currentEntries, table.lookupRecords, lookups);
        }
        for (const coverage of table.lookaheadCoverage) {
            currentEntries = helper_1.processLookaheadPosition(coverage_1.listGlyphsByIndex(coverage).map(glyph => glyph.glyphId), currentEntries);
        }
        for (const coverage of table.backtrackCoverage) {
            currentEntries = helper_1.processBacktrackPosition(coverage_1.listGlyphsByIndex(coverage).map(glyph => glyph.glyphId), currentEntries);
        }
        // When we get to the end, all of the entries we've accumulated
        // should have a lookup defined
        for (const { entry, substitutions } of currentEntries) {
            entry.lookup = {
                substitutions,
                index: tableIndex,
                subIndex: 0,
                length: table.inputCoverage.length,
                contextRange: [
                    -1 * table.backtrackCoverage.length,
                    table.inputCoverage.length + table.lookaheadCoverage.length
                ]
            };
        }
    }
    return result;
}
exports.default = buildTree;
//# sourceMappingURL=6-3.js.map