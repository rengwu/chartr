/// <reference types="node" />
export interface PostTable {
    version: number;
    italicAngle: number;
    underlinePosition: number;
    underlineThickness: number;
    isFixedPitch: number;
    minMemType42: number;
    maxMemType42: number;
    minMemType1: number;
    maxMemType1: number;
}
export default function parsePostTable(data: Buffer): PostTable;
