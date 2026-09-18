"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Get the number of the class to which the glyph belongs, or null if it doesn't
 * belong to any of them.
 *
 * @param table JSON representation of the class def table
 * @param glyphId Index of the glyph to look for
 */
function getGlyphClass(table, glyphId) {
    switch (table.format) {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table-format-2
        case 2:
            if (Array.isArray(glyphId)) {
                return getRangeGlyphClass(table, glyphId);
            }
            else {
                return new Map([[
                        glyphId,
                        getIndividualGlyphClass(table, glyphId)
                    ]]);
            }
        // https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table-format-1
        default:
            return new Map([[glyphId, null]]);
    }
}
exports.default = getGlyphClass;
function getRangeGlyphClass(table, glyphId) {
    let classStart = glyphId[0];
    let currentClass = getIndividualGlyphClass(table, classStart);
    let search = glyphId[0] + 1;
    const result = new Map();
    while (search < glyphId[1]) {
        const clazz = getIndividualGlyphClass(table, search);
        if (clazz !== currentClass) {
            if (search - classStart <= 1) {
                result.set(classStart, currentClass);
            }
            else {
                result.set([classStart, search], currentClass);
            }
        }
        search++;
    }
    if (search - classStart <= 1) {
        result.set(classStart, currentClass);
    }
    else {
        result.set([classStart, search], currentClass);
    }
    return result;
}
function getIndividualGlyphClass(table, glyphId) {
    for (const range of table.ranges) {
        if (range.start <= glyphId && range.end >= glyphId) {
            return range.classId;
        }
    }
    return null;
}
function listClassGlyphs(table, index) {
    switch (table.format) {
        case 2:
            const results = [];
            for (const range of table.ranges) {
                if (range.classId !== index) {
                    continue;
                }
                if (range.end === range.start) {
                    results.push(range.start);
                }
                else {
                    results.push([range.start, range.end + 1]);
                }
            }
            return results;
        default:
            return [];
    }
}
exports.listClassGlyphs = listClassGlyphs;
//# sourceMappingURL=classDef.js.map