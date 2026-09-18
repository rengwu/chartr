"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const substitution_1 = require("./substitution");
function processInputPosition(glyphs, position, currentEntries, lookupRecords, lookups) {
    const nextEntries = [];
    for (const currentEntry of currentEntries) {
        currentEntry.entry.forward = {
            individual: {},
            range: []
        };
        for (const glyph of glyphs) {
            nextEntries.push(...getInputTree(currentEntry.entry.forward, lookupRecords, lookups, position, glyph).map(({ entry, substitution }) => ({
                entry,
                substitutions: [...currentEntry.substitutions, substitution]
            })));
        }
    }
    return nextEntries;
}
exports.processInputPosition = processInputPosition;
function processLookaheadPosition(glyphs, currentEntries) {
    const nextEntries = [];
    for (const currentEntry of currentEntries) {
        for (const glyph of glyphs) {
            const entry = {};
            if (!currentEntry.entry.forward) {
                currentEntry.entry.forward = {
                    individual: {},
                    range: []
                };
            }
            nextEntries.push({
                entry,
                substitutions: currentEntry.substitutions
            });
            if (Array.isArray(glyph)) {
                currentEntry.entry.forward.range.push({
                    entry,
                    range: glyph
                });
            }
            else {
                currentEntry.entry.forward.individual[glyph] = entry;
            }
        }
    }
    return nextEntries;
}
exports.processLookaheadPosition = processLookaheadPosition;
function processBacktrackPosition(glyphs, currentEntries) {
    const nextEntries = [];
    for (const currentEntry of currentEntries) {
        for (const glyph of glyphs) {
            const entry = {};
            if (!currentEntry.entry.reverse) {
                currentEntry.entry.reverse = {
                    individual: {},
                    range: []
                };
            }
            nextEntries.push({
                entry,
                substitutions: currentEntry.substitutions
            });
            if (Array.isArray(glyph)) {
                currentEntry.entry.reverse.range.push({
                    entry,
                    range: glyph
                });
            }
            else {
                currentEntry.entry.reverse.individual[glyph] = entry;
            }
        }
    }
    return nextEntries;
}
exports.processBacktrackPosition = processBacktrackPosition;
function getInputTree(tree, substitutions, lookups, inputIndex, glyphId) {
    const result = [];
    if (!Array.isArray(glyphId)) {
        tree.individual[glyphId] = {};
        result.push({
            entry: tree.individual[glyphId],
            substitution: getSubstitutionAtPosition(substitutions, lookups, inputIndex, glyphId)
        });
    }
    else {
        const subs = getSubstitutionAtPositionRange(substitutions, lookups, inputIndex, glyphId);
        for (const [range, substitution] of subs) {
            const entry = {};
            if (Array.isArray(range)) {
                tree.range.push({ range, entry });
            }
            else {
                tree.individual[range] = {};
            }
            result.push({ entry, substitution });
        }
    }
    return result;
}
exports.getInputTree = getInputTree;
function getSubstitutionAtPositionRange(substitutions, lookups, index, range) {
    for (const substitution of substitutions.filter(s => s.sequenceIndex === index)) {
        for (const substitutionTable of lookups[substitution.lookupListIndex].subtables) {
            const sub = substitution_1.getRangeSubstitutionGlyphs(substitutionTable, range);
            if (!Array.from(sub.values()).every(val => val !== null)) {
                return sub;
            }
        }
    }
    return new Map([[range, null]]);
}
function getSubstitutionAtPosition(substitutions, lookups, index, glyphId) {
    for (const substitution of substitutions.filter(s => s.sequenceIndex === index)) {
        for (const substitutionTable of lookups[substitution.lookupListIndex].subtables) {
            const sub = substitution_1.getIndividualSubstitutionGlyph(substitutionTable, glyphId);
            if (sub !== null) {
                return sub;
            }
        }
    }
    return null;
}
//# sourceMappingURL=helper.js.map