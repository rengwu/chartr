"use strict";
// This file is modified from opentype.js. All credit for the capabilities
// provided herein goes to that project and its maintainers. The project can be
// found at https://github.com/nodebox/opentype.js
Object.defineProperty(exports, "__esModule", { value: true });
// Parse the OS/2 and Windows metrics `OS/2` table
function parseOS2Table(data) {
    // The OS/2 table must be at least 78 bytes long
    if (data.length < 78) {
        return undefined;
    }
    const os2 = {
        version: data.readUInt16BE(0),
        xAvgCharWidth: data.readUInt16BE(2),
        usWeightClass: data.readUInt16BE(4),
        usWidthClass: data.readUInt16BE(6),
        fsType: data.readUInt16BE(8),
        ySubscriptXSize: data.readInt16BE(10),
        ySubscriptYSize: data.readInt16BE(12),
        ySubscriptXOffset: data.readInt16BE(14),
        ySubscriptYOffset: data.readInt16BE(16),
        ySuperscriptXSize: data.readInt16BE(18),
        ySuperscriptYSize: data.readInt16BE(20),
        ySuperscriptXOffset: data.readInt16BE(22),
        ySuperscriptYOffset: data.readInt16BE(24),
        yStrikeoutSize: data.readInt16BE(26),
        yStrikeoutPosition: data.readInt16BE(28),
        sFamilyClass: data.readInt16BE(30),
        panose: [
            data.readUInt8(32),
            data.readUInt8(33),
            data.readUInt8(34),
            data.readUInt8(35),
            data.readUInt8(36),
            data.readUInt8(37),
            data.readUInt8(38),
            data.readUInt8(39),
            data.readUInt8(40),
            data.readUInt8(41)
        ],
        ulUnicodeRange1: data.readUInt32BE(42),
        ulUnicodeRange2: data.readUInt32BE(46),
        ulUnicodeRange3: data.readUInt32BE(50),
        ulUnicodeRange4: data.readUInt32BE(54),
        achVendID: String.fromCharCode(data.readUInt8(58), data.readUInt8(59), data.readUInt8(60), data.readUInt8(61)),
        fsSelection: data.readUInt16BE(62),
        usFirstCharIndex: data.readUInt16BE(64),
        usLastCharIndex: data.readUInt16BE(66),
        sTypoAscender: data.readInt16BE(68),
        sTypoDescender: data.readInt16BE(70),
        sTypoLineGap: data.readInt16BE(72),
        usWinAscent: data.readUInt16BE(74),
        usWinDescent: data.readUInt16BE(76)
    };
    if (os2.version >= 1 && data.length >= 86) {
        os2.ulCodePageRange1 = data.readUInt32BE(78);
        os2.ulCodePageRange2 = data.readUInt32BE(82);
    }
    if (os2.version >= 2 && data.length >= 96) {
        os2.sxHeight = data.readInt16BE(86);
        os2.sCapHeight = data.readInt16BE(88);
        os2.usDefaultChar = data.readUInt16BE(90);
        os2.usBreakChar = data.readUInt16BE(92);
        os2.usMaxContent = data.readUInt16BE(94);
    }
    return os2;
}
exports.default = parseOS2Table;
//# sourceMappingURL=os2.js.map