import { formatFixed, formatLongDateTime } from './utility';

// This file is modified from opentype.js. All credit for the capabilities
// provided herein goes to that project and its maintainers. The project can be
// found at https://github.com/nodebox/opentype.js

// The `head` table contains global information about the font.
// https://www.microsoft.com/typography/OTSPEC/head.htm

export interface HeadTable {
    version: number;
    fontRevision: number;
    checkSumAdjustment: number;
    magicNumber: number;
    flags: number;
    unitsPerEm: number;
    created: number;
    modified: number;
    xMin: number;
    yMin: number;
    xMax: number;
    yMax: number;
    macStyle: number;
    lowestRecPPEM: number;
    fontDirectionHint: number;
    indexToLocFormat: number;
    glyphDataFormat: number;
}

// Parse the header `head` table
export default function parseHeadTable(data: Buffer): HeadTable {
    return {
        version: formatFixed(data.readUInt16BE(0), data.readUInt16BE(2)),
        fontRevision: formatFixed(data.readUInt16BE(4), data.readUInt16BE(6)),
        checkSumAdjustment: data.readUInt32BE(8),
        magicNumber: data.readUInt32BE(12),
        flags: data.readUInt16BE(16),
        unitsPerEm: data.readUInt16BE(18),
        created: formatLongDateTime(data.readUInt32BE(20), data.readUInt32BE(24)),
        modified: formatLongDateTime(data.readUInt32BE(28), data.readUInt32BE(32)),
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
