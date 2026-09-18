/// <reference types="node" />
export interface HeadTable {
    version: number;
    fontRevision: number;
    checkSumAdjustment: number;
    magicNumber: number;
    flags: number;
    unitsPerEm: number;
    created: number;
    modified: number;
    xMin: number;
    yMin: number;
    xMax: number;
    yMax: number;
    macStyle: number;
    lowestRecPPEM: number;
    fontDirectionHint: number;
    indexToLocFormat: number;
    glyphDataFormat: number;
}
export default function parseHeadTable(data: Buffer): HeadTable;
