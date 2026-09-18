"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const coverage_1 = require("./coverage");
/**
 * Get the substitution glyph for the givne glyph, or null if the glyph was not
 * found in the table.
 *
 * @param table JSON representation of the substitution table
 * @param glyphId The index of the glpyh to find substitutions for
 */
function getRangeSubstitutionGlyphs(table, glyphId) {
    let replacementStart = glyphId[0];
    let currentReplacement = getIndividualSubstitutionGlyph(table, replacementStart);
    let search = glyphId[0] + 1;
    const result = new Map();
    while (search < glyphId[1]) {
        const sub = getIndividualSubstitutionGlyph(table, search);
        if (sub !== currentReplacement) {
            if (search - replacementStart <= 1) {
                result.set(replacementStart, currentReplacement);
            }
            else {
                result.set([replacementStart, search], currentReplacement);
            }
        }
        search++;
    }
    if (search - replacementStart <= 1) {
        result.set(replacementStart, currentReplacement);
    }
    else {
        result.set([replacementStart, search], currentReplacement);
    }
    return result;
}
exports.getRangeSubstitutionGlyphs = getRangeSubstitutionGlyphs;
function getIndividualSubstitutionGlyph(table, glyphId) {
    const coverageIndex = coverage_1.default(table.coverage, glyphId);
    // istanbul ignore next - invalid font
    if (coverageIndex === null) {
        return null;
    }
    switch (table.substFormat) {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#11-single-substitution-format-1
        case 1:
            // TODO: determine if there's a rhyme or reason to the 16-bit
            // wraparound and if it can ever be a different number
            return (glyphId + table.deltaGlyphId) % (2 ** 16);
        // https://docs.microsoft.com/en-us/typography/opentype/spec/gsub#12-single-substitution-format-2
        case 2:
            // tslint:disable-next-line
            return table.substitute[coverageIndex] != null
                ? table.substitute[coverageIndex]
                : null;
    }
}
exports.getIndividualSubstitutionGlyph = getIndividualSubstitutionGlyph;
//# sourceMappingURL=substitution.js.map