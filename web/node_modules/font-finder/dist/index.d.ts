import * as extract from './extract';
export { Type, Style } from './extract';
/**
 * Metadata about a single variant of a font.
 */
export interface Font {
    /**
     * Absolute path to the file containing the font
     */
    path: string;
    /**
     * The type of font (serif, sans-serif, monospace, etc.)
     */
    type: extract.Type;
    /**
     * A number between 1 and 1000 represent the thickness of the font strokes
     */
    weight: number;
    /**
     * Style information about the font, such as whether it's bold, italic,
     * oblique, or some combination of the above.
     */
    style: extract.Style;
}
/**
 * Font metadata including its name
 */
export interface NamedFont extends Font {
    /**
     * The font's family name
     */
    name: string;
}
/**
 * Object mapping font family names to a list with metadata for each variant
 */
export interface FontList {
    [fontName: string]: Font[];
}
/**
 * Options configuring the list operation
 */
export interface ListOptions {
    /**
     * Maximum number of fonts to process concurently. You can tweak this to get
     * the maximum performance out of the call. Default: 4
     */
    concurrency?: number;
    /**
     * The language to use when fetching font naming information. Default: 'en'
     */
    language?: string;
    /**
     * Function to call on font loading error. Default: null
     */
    onFontError?: ((path: string, err: Error) => void) | null;
}
/**
 * Options configuring the list operation
 */
export interface GetOptions {
    /**
     * The language to use when fetching font naming information. Default: 'en'
     */
    language?: string;
}
/**
 * Retrieve metadata for all fonts installed on the system.
 *
 * @param options Options to configure font retrieval
 */
export declare function list(options?: ListOptions): Promise<FontList>;
/**
 * Lists all variants found for the provided font family. If no variants are
 * found, an empty array is returned.
 *
 * @param name The name of the font family to retrieve
 * @param options Options to configure font retrieval
 */
export declare function listVariants(name: string, options?: ListOptions): Promise<Font[]>;
/**
 * Gets metadata for a single font file, returning metadata for the first
 * font variant found in the file. If there is an error extracting the font, an
 * error is thrown (unlike in list, where the font is simply ignored).
 *
 * @param path Absolute path to the file to retrieve
 * @param options Options to configure font retrieval
 */
export declare function get(path: string, options?: GetOptions): Promise<NamedFont>;
