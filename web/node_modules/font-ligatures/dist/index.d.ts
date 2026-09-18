import { Font, LigatureData, Options } from './types';
/**
 * Load the font with the given name. The returned value can be used to find
 * ligatures for the font.
 *
 * @param name Font family name for the font to load
 */
export declare function load(name: string, options?: Options): Promise<Font>;
/**
 * Load the font at the given file path. The returned value can be used to find
 * ligatures for the font.
 *
 * @param path Path to the file containing the font
 */
export declare function loadFile(path: string, options?: Options): Promise<Font>;
/**
 * Load the font from it's binary data. The returned value can be used to find
 * ligatures for the font.
 *
 * @param buffer ArrayBuffer of the font to load
 */
export declare function loadBuffer(buffer: ArrayBuffer, options?: Options): Font;
export { Font, LigatureData, Options };
