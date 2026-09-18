"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const utility_1 = require("./utility");
// Parse the PostScript `post` table. We don't bother with version-specific data
// because it doesn't impact any of our computations
function parsePostTable(data) {
    return {
        version: utility_1.formatFixed(data.readUInt16BE(0), data.readUInt16BE(2)),
        italicAngle: utility_1.formatFixed(data.readUInt16BE(4), data.readUInt16BE(6)),
        underlinePosition: data.readInt16BE(8),
        underlineThickness: data.readInt16BE(10),
        isFixedPitch: data.readUInt32BE(12),
        minMemType42: data.readUInt32BE(16),
        maxMemType42: data.readUInt32BE(20),
        minMemType1: data.readUInt32BE(24),
        maxMemType1: data.readUInt32BE(28)
    };
}
exports.default = parsePostTable;
//# sourceMappingURL=post.js.map