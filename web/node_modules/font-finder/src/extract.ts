import * as os from 'os';

import { FontData } from './parse';

export enum Type {
    Serif = 'serif',
    SansSerif = 'sansSerif',
    Monospace = 'monospace',
    Cursive = 'cursive',
    Unknown = 'unknown'
}

export enum Style {
    Regular = 'regular',
    Italic = 'italic',
    Oblique = 'oblique',
    Bold = 'bold',
    BoldItalic = 'boldItalic',
    BoldOblique = 'boldOblique',
    Other = 'other'
}

const standardEndings = [
    ' Regular',
    ' Bold',
    ' Bold Italic',
    ' Bold Oblique',
    ' Italic',
    ' Oblique'
];

export function name(fontData: FontData, language: string): string {
    const family = fontData.names.preferredFamily && fontData.names.preferredFamily[language]
        ? fontData.names.preferredFamily[language]
        : fontData.names.fontFamily[language];

    // On Windows, if the full font name doesn't end with one of the standard
    // forms, the full name is needed to identify the font. Notably, this is not
    // the same thing as the subfamily matching it, as with 'Roboto Thin Italic'
    // where the subfamily is 'Thin Italic'. In this case, the 'Italic' should
    // be removed, but not the 'Thin'.
    // TODO: actually, 'Roboto' and 'Roboto Thin' seem to both work. This needs
    // more work to figure out the exact logic
    if (os.platform() === 'win32') {
        const subfamily = fontData.names.preferredSubfamily && fontData.names.preferredSubfamily[language]
            ? fontData.names.preferredSubfamily[language]
            : fontData.names.fontSubfamily[language];
        const fullName = `${family} ${subfamily}`;

        let endIndex = -1;
        for (const end of standardEndings) {
            const index = fullName.lastIndexOf(end);
            if (index !== -1) {
                endIndex = index;
                break;
            }
        }

        if (endIndex !== -1) {
            return fullName.substring(0, endIndex);
        }

        return fullName;
    }

    return family;
}

export function type(fontData: FontData): Type {
    if (fontData.os2) {
        // Panose specification: https://monotype.github.io/panose/pan1.htm
        switch (fontData.os2.panose[0]) {
            case 2:
                // https://monotype.github.io/panose/pan2.htm#_Toc380547256
                if (fontData.os2.panose[3] === 9) {
                    return Type.Monospace;
                }

                // https://monotype.github.io/panose/pan2.htm#Sec2SerifStyle
                if (
                    fontData.os2.panose[1] >= 11 &&
                    fontData.os2.panose[1] <= 15 ||
                    fontData.os2.panose[1] === 0
                ) {
                    return Type.SansSerif;
                }

                return Type.Serif;
            case 3:
                return Type.Cursive;
        }
    } else if (fontData.post && fontData.post.isFixedPitch) {
        return Type.Monospace;
    }

    // TODO: better classification
    return Type.Unknown;
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/os2#fsselection
export function style(fontData: FontData): Style {
    // If we don't have an OS/2 or head table, there's no good way to figure out
    // what's in the font
    if (!fontData.os2 && !fontData.head) {
        return Style.Other;
    }

    const bold = fontData.os2
        ? fontData.os2.fsSelection & 0x20       // OS/2: fsSelection bit 5
        : fontData.head!.macStyle & 0x01;       // head: macStyle bit 0
    const italic = fontData.os2
        ? fontData.os2.fsSelection & 0x01       // OS/2: fsSelection bit 0
        : fontData.post
            ? fontData.post.italicAngle < 0     // post: negative italicAngle
            : fontData.head!.macStyle & 0x02;   // head: macStyle bit 1
    const oblique = fontData.os2
        ? fontData.os2.fsSelection & 0x200      // OS/2: fsSelection bit 9
        : fontData.post
            ? fontData.post.italicAngle > 0     // post: positive italicAngle
            : 0;                                // head: N/A
    const regular = fontData.os2
        ? fontData.os2.fsSelection & 0x140      // OS/2: fsSelection bit 6 or 8 (WWS)
        : 1;                                    // head: N/A (assume yes for fallback)

    if (bold) {
        // Oblique has to come before italic for it to get picked up
        if (oblique) {
            return Style.BoldOblique;
        }

        if (italic) {
            return Style.BoldItalic;
        }

        return Style.Bold;
    }

    // Oblique has to come before italic for it to get picked up
    if (oblique) {
        return Style.Oblique;
    }

    if (italic) {
        return Style.Italic;
    }

    if (regular) {
        return Style.Regular;
    }

    // TODO: better classification
    return Style.Other;
}

const boldStyles = [Style.Bold, Style.BoldItalic, Style.BoldOblique];

export function weight(fontData: FontData): number {
    if (fontData.os2) {
        // Use the OS/2 weight class if available
        return fontData.os2.usWeightClass;
    } else if (boldStyles.includes(style(fontData))) {
        // Assume 700 if the font is a bold font
        return 700;
    } else {
        // Assume the standard 400 if all else fails
        return 400;
    }
}
