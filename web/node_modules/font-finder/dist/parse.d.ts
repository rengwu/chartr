import { NameTable } from './tables/name';
import { OS2Table } from './tables/os2';
import { HeadTable } from './tables/head';
import { PostTable } from './tables/post';
export declare type FontData = {
    names: NameTable;
    os2?: OS2Table;
    head?: HeadTable;
    post?: PostTable;
};
/**
 * Loads the bare minimum information needed to retrieve the metadata that we
 * want, streaming the data from the file until we've found everything we need.
 *
 * @param filePath Absolute path to the font to load
 */
export default function parseFont(filePath: string): Promise<FontData>;
