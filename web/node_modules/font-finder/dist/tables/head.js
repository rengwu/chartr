"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const utility_1 = require("./utility");
// Parse the header `head` table
function parseHeadTable(data) {
    return {
        version: utility_1.formatFixed(data.readUInt16BE(0), data.readUInt16BE(2)),
        fontRevision: utility_1.formatFixed(data.readUInt16BE(4), data.readUInt16BE(6)),
        checkSumAdjustment: data.readUInt32BE(8),
        magicNumber: data.readUInt32BE(12),
        flags: data.readUInt16BE(16),
        unitsPerEm: data.readUInt16BE(18),
        created: utility_1.formatLongDateTime(data.readUInt32BE(20), data.readUInt32BE(24)),
        modified: utility_1.formatLongDateTime(data.readUInt32BE(28), data.readUInt32BE(32)),
        xMin: data.readInt16BE(36),
        yMin: data.readInt16BE(38),
        xMax: data.readInt16BE(40),
        yMax: data.readInt16BE(42),
        macStyle: data.readUInt16BE(44),
        lowestRecPPEM: data.readUInt16BE(46),
        fontDirectionHint: data.readInt16BE(48),
        indexToLocFormat: data.readInt16BE(50),
        glyphDataFormat: data.readInt16BE(52)
    };
}
exports.default = parseHeadTable;
//# sourceMappingURL=head.js.map