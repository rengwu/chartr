"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
function flatten(tree) {
    const result = {};
    for (const [glyphId, entry] of Object.entries(tree.individual)) {
        result[glyphId] = flattenEntry(entry);
    }
    for (const { range, entry } of tree.range) {
        const flattened = flattenEntry(entry);
        for (let glyphId = range[0]; glyphId < range[1]; glyphId++) {
            result[glyphId] = flattened;
        }
    }
    return result;
}
exports.default = flatten;
function flattenEntry(entry) {
    const result = {};
    if (entry.forward) {
        result.forward = flatten(entry.forward);
    }
    if (entry.reverse) {
        result.reverse = flatten(entry.reverse);
    }
    if (entry.lookup) {
        result.lookup = entry.lookup;
    }
    return result;
}
//# sourceMappingURL=flatten.js.map