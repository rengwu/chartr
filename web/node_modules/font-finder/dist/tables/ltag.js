"use strict";
// This file is modified from opentype.js. All credit for the capabilities
// provided herein goes to that project and its maintainers. The project can be
// found at https://github.com/nodebox/opentype.js
Object.defineProperty(exports, "__esModule", { value: true });
// The `ltag` table stores IETF BCP-47 language tags. It allows supporting
// languages for which TrueType does not assign a numeric code.
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6ltag.html
// http://www.w3.org/International/articles/language-tags/
// http://www.iana.org/assignments/language-subtag-registry/language-subtag-registry
function parseLtagTable(data) {
    const tableVersion = data.readUInt32BE(0);
    if (tableVersion !== 1) {
        throw new Error('Unsupported ltag table version.');
    }
    // The 'ltag' specification does not define any flags; skip the field.
    const numTags = data.readUInt32BE(8);
    const tags = [];
    for (let i = 0; i < numTags; i++) {
        let tag = '';
        const offset = data.readUInt16BE(12 + i * 4);
        const length = data.readUInt16BE(14 + i * 4);
        for (let j = offset; j < offset + length; ++j) {
            tag += String.fromCharCode(data.readInt8(j));
        }
        tags.push(tag);
    }
    return tags;
}
exports.default = parseLtagTable;
//# sourceMappingURL=ltag.js.map