export function formatFixed(whole: number, fraction: number) {
    return whole + fraction / 2 ** 16;
}

export function formatLongDateTime(high: number, low: number) {
    // OpenType dates are since 1904. We make them since 1970 to align with unix
    // and multiply by 1000 to make it a millisecond time like the rest of
    // Javascript
    return ((high * 2 ** 32) + low - 2082844800) * 1000;
}
