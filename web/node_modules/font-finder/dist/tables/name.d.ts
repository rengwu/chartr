/// <reference types="node" />
export interface NameTable {
    [name: string]: {
        [language: string]: string;
    };
}
export default function parseNameTable(data: Buffer, ltag: string[]): NameTable;
