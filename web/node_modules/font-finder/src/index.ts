import getSystemFonts from 'get-system-fonts';

import parse, { FontData } from './parse';
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
export async function list(options?: ListOptions): Promise<FontList> {
    const opts: Required<ListOptions> = {
        concurrency: 4,
        language: 'en',
        onFontError: null,
        ...options
    };

    // TODO: support woff, woff2, ttc
    const files = await getSystemFonts({ extensions: ['ttf', 'otf'] });

    // Process each font in parallel, swallowing any errors found along the way.
    const results = await parallelize(
        async (file): Promise<NamedFont | undefined> => {
            try {
                const fontData = await parse(file);
                return getMetadata(file, fontData, opts.language);
            } catch (e) {
                if (opts.onFontError) {
                    opts.onFontError(file, e);
                }
            }
        },
        files,
        opts.concurrency
    );

    // Group the fonts by their font family
    const fonts: FontList = {};
    for (const { name, ...font } of results.filter(font => font) as NamedFont[]) {
        if (!fonts[name]) {
            fonts[name] = [];
        }

        fonts[name].push(font);
    }

    return fonts;
}

/**
 * Lists all variants found for the provided font family. If no variants are
 * found, an empty array is returned.
 *
 * @param name The name of the font family to retrieve
 * @param options Options to configure font retrieval
 */
export async function listVariants(name: string, options?: ListOptions): Promise<Font[]> {
    const fonts = await list(options);

    return fonts[name] || [];
}

/**
 * Gets metadata for a single font file, returning metadata for the first
 * font variant found in the file. If there is an error extracting the font, an
 * error is thrown (unlike in list, where the font is simply ignored).
 *
 * @param path Absolute path to the file to retrieve
 * @param options Options to configure font retrieval
 */
export async function get(path: string, options?: GetOptions): Promise<NamedFont> {
    const opts: Required<GetOptions> = {
        language: 'en',
        ...options
    };

    const fontData = await parse(path);
    return getMetadata(path, fontData, opts.language);
}

/**
 * Extracts font metadata, given the font information.
 *
 * @param path Absolute path to the font file
 * @param fontData Table data for the font
 * @param language Language to use when resolving names
 */
function getMetadata(path: string, fontData: FontData, language: string): NamedFont {
    return {
        name: extract.name(fontData, language),
        path,
        type: extract.type(fontData),
        weight: extract.weight(fontData),
        style: extract.style(fontData)
    };
}

/**
 * Runs an asynchronous operation against a list of inputs, capping the
 * concurrency of execution at the provided number.
 *
 * @param operation Function to run with each input
 * @param data Array of inputs to be applied to the function. Inputs are started
 *             in array order
 * @param concurrency The maximum number of operations to run simultaneously
 */
async function parallelize<S, T>(operation: (input: S) => Promise<T>, data: S[], concurrency: number): Promise<T[]> {
    const results: T[] = [];
    let index = 0;

    const wrapper = async (i: number) => {
        results.push(await operation(data[i]));
        if (index < data.length) {
            await wrapper(index++);
        }
    };

    const promises: Promise<void>[] = [];
    for (; index < data.length && index < concurrency; index++) {
        promises.push(wrapper(index));
    }

    await Promise.all(promises);
    return results;
}
