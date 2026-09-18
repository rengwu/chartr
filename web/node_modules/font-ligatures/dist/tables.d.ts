export declare type SubstitutionTable = SubstitutionTable.Format1 | SubstitutionTable.Format2;
export declare namespace SubstitutionTable {
    interface Format1 {
        substFormat: 1;
        coverage: CoverageTable;
        deltaGlyphId: number;
    }
    interface Format2 {
        substFormat: 2;
        coverage: CoverageTable;
        substitute: number[];
    }
}
export declare type CoverageTable = CoverageTable.Format1 | CoverageTable.Format2;
export declare namespace CoverageTable {
    interface Format1 {
        format: 1;
        glyphs: number[];
    }
    interface Format2 {
        format: 2;
        ranges: {
            start: number;
            end: number;
            index: number;
        }[];
    }
}
export declare type ChainingContextualSubstitutionTable = ChainingContextualSubstitutionTable.Format1 | ChainingContextualSubstitutionTable.Format2 | ChainingContextualSubstitutionTable.Format3;
export declare namespace ChainingContextualSubstitutionTable {
    interface Format1 {
        substFormat: 1;
        coverage: CoverageTable;
        chainRuleSets: ChainSubRuleTable[][];
    }
    interface Format2 {
        substFormat: 2;
        coverage: CoverageTable;
        backtrackClassDef: ClassDefTable;
        inputClassDef: ClassDefTable;
        lookaheadClassDef: ClassDefTable;
        chainClassSet: (null | ChainSubClassRuleTable[])[];
    }
    interface Format3 {
        substFormat: 3;
        backtrackCoverage: CoverageTable[];
        inputCoverage: CoverageTable[];
        lookaheadCoverage: CoverageTable[];
        lookupRecords: SubstitutionLookupRecord[];
    }
}
export interface ReverseChainingContextualSingleSubstitutionTable {
    substFormat: 1;
    coverage: CoverageTable;
    backtrackCoverage: CoverageTable[];
    lookaheadCoverage: CoverageTable[];
    substitutes: number[];
}
export declare type ClassDefTable = ClassDefTable.Format2;
export declare namespace ClassDefTable {
    interface Format2 {
        format: 2;
        ranges: {
            start: number;
            end: number;
            classId: number;
        }[];
    }
}
export interface SubstitutionLookupRecord {
    sequenceIndex: number;
    lookupListIndex: number;
}
export declare type ChainSubRuleTable = ChainSubClassRuleTable;
export interface ChainSubClassRuleTable {
    backtrack: number[];
    input: number[];
    lookahead: number[];
    lookupRecords: SubstitutionLookupRecord[];
}
export declare type Lookup = Lookup.Type1 | Lookup.Type6 | Lookup.Type8;
export declare namespace Lookup {
    interface Type1 {
        lookupType: 1;
        lookupFlag: number;
        subtables: SubstitutionTable[];
    }
    interface Type6 {
        lookupType: 6;
        lookupFlag: number;
        subtables: ChainingContextualSubstitutionTable[];
    }
    interface Type8 {
        lookupType: 8;
        lookupFlag: number;
        subtables: ReverseChainingContextualSingleSubstitutionTable[];
    }
}
