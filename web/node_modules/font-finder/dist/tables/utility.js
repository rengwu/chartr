"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
function formatFixed(whole, fraction) {
    return whole + fraction / 2 ** 16;
}
exports.formatFixed = formatFixed;
function formatLongDateTime(high, low) {
    // OpenType dates are since 1904. We make them since 1970 to align with unix
    // and multiply by 1000 to make it a millisecond time like the rest of
    // Javascript
    return ((high * 2 ** 32) + low - 2082844800) * 1000;
}
exports.formatLongDateTime = formatLongDateTime;
//# sourceMappingURL=utility.js.map