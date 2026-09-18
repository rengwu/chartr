"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const merge_1 = require("../merge");
const coverage_1 = require("./coverage");
const classDef_1 = require("./classDef");
const helper_1 = require("./helper");
/**
 * Build lookup tree for GSUB lookup table 6, format 2.
 * https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#62-chaining-context-substitution-format-2-class-based-glyph-contexts
 *
 * @param table JSON representation of the table
 * @param lookups List of lookup tables
 * @param tableIndex Index of this table in the overall lookup
 */
function buildTree(table, lookups, tableIndex) {
    const results = [];
    const firstGlyphs = coverage_1.listGlyphsByIndex(table.coverage);
    for (const { glyphId } of firstGlyphs) {
        const firstInputClass = classDef_1.default(table.inputClassDef, glyphId);
        for (const [glyphId, inputClass] of firstInputClass.entries()) {
            // istanbul ignore next - invalid font
            if (inputClass === null) {
                continue;
            }
            const classSet = table.chainClassSet[inputClass];
            // If the class set is null there's nothing to do with this table.
            if (!classSet) {
                continue;
            }
            for (const [subIndex, subTable] of classSet.entries()) {
                const result = {
                    individual: {},
                    range: []
                };
                let currentEntries = helper_1.getInputTree(result, subTable.lookupRecords, lookups, 0, glyphId).map(({ entry, substitution }) => ({ entry, substitutions: [substitution] }));
                for (const [index, classNum] of subTable.input.entries()) {
                    currentEntries = helper_1.processInputPosition(classDef_1.listClassGlyphs(table.inputClassDef, classNum), index + 1, currentEntries, subTable.lookupRecords, lookups);
                }
                for (const classNum of subTable.lookahead) {
                    currentEntries = helper_1.processLookaheadPosition(classDef_1.listClassGlyphs(table.lookaheadClassDef, classNum), currentEntries);
                }
                for (const classNum of subTable.backtrack) {
                    currentEntries = helper_1.processBacktrackPosition(classDef_1.listClassGlyphs(table.backtrackClassDef, classNum), currentEntries);
                }
                // When we get to the end, all of the entries we've accumulated
                // should have a lookup defined
                for (const { entry, substitutions } of currentEntries) {
                    entry.lookup = {
                        substitutions,
                        index: tableIndex,
                        subIndex,
                        length: subTable.input.length + 1,
                        contextRange: [
                            -1 * subTable.backtrack.length,
                            1 + subTable.input.length + subTable.lookahead.length
                        ]
                    };
                }
                results.push(result);
            }
        }
    }
    return merge_1.default(results);
}
exports.default = buildTree;
//# sourceMappingURL=6-2.js.map