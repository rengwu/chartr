import { FontData } from './parse';
export declare enum Type {
    Serif = "serif",
    SansSerif = "sansSerif",
    Monospace = "monospace",
    Cursive = "cursive",
    Unknown = "unknown"
}
export declare enum Style {
    Regular = "regular",
    Italic = "italic",
    Oblique = "oblique",
    Bold = "bold",
    BoldItalic = "boldItalic",
    BoldOblique = "boldOblique",
    Other = "other"
}
export declare function name(fontData: FontData, language: string): string;
export declare function type(fontData: FontData): Type;
export declare function style(fontData: FontData): Style;
export declare function weight(fontData: FontData): number;
