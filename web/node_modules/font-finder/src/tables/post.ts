import { formatFixed } from './utility';

// This file is modified from opentype.js. All credit for the capabilities
// provided herein goes to that project and its maintainers. The project can be
// found at https://github.com/nodebox/opentype.js

// The `post` table stores additional PostScript information, such as glyph names.
// https://www.microsoft.com/typography/OTSPEC/post.htm

export interface PostTable {
    version: number;
    italicAngle: number;
    underlinePosition: number;
    underlineThickness: number;
    isFixedPitch: number;
    minMemType42: number;
    maxMemType42: number;
    minMemType1: number;
    maxMemType1: number;
}

// Parse the PostScript `post` table. We don't bother with version-specific data
// because it doesn't impact any of our computations
export default function parsePostTable(data: Buffer): PostTable {
    return {
        version: formatFixed(data.readUInt16BE(0), data.readUInt16BE(2)),
        italicAngle: formatFixed(data.readUInt16BE(4), data.readUInt16BE(6)),
        underlinePosition: data.readInt16BE(8),
        underlineThickness: data.readInt16BE(10),
        isFixedPitch: data.readUInt32BE(12),
        minMemType42: data.readUInt32BE(16),
        maxMemType42: data.readUInt32BE(20),
        minMemType1: data.readUInt32BE(24),
        maxMemType1: data.readUInt32BE(28)
    };
}
