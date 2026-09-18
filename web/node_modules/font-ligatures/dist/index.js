"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const opentype = require("opentype.js");
const lru = require("lru-cache");
const merge_1 = require("./merge");
const walk_1 = require("./walk");
const mergeRange_1 = require("./mergeRange");
const _6_1_1 = require("./processors/6-1");
const _6_2_1 = require("./processors/6-2");
const _6_3_1 = require("./processors/6-3");
const _8_1_1 = require("./processors/8-1");
const flatten_1 = require("./flatten");
class FontImpl {
    constructor(font, options) {
        this._lookupTrees = [];
        this._glyphLookups = {};
        this._font = font;
        if (options.cacheSize > 0) {
            this._cache = new lru({
                max: options.cacheSize,
                length: ((val, key) => key.length)
            });
        }
        const caltFeatures = this._font.tables.gsub && this._font.tables.gsub.features.filter(f => f.tag === 'calt') || [];
        const lookupIndices = caltFeatures
            .reduce((acc, val) => [...acc, ...val.feature.lookupListIndexes], []);
        const allLookups = this._font.tables.gsub && this._font.tables.gsub.lookups || [];
        const lookupGroups = allLookups.filter((l, i) => lookupIndices.some(idx => idx === i));
        for (const [index, lookup] of lookupGroups.entries()) {
            const trees = [];
            switch (lookup.lookupType) {
                case 6:
                    for (const [index, table] of lookup.subtables.entries()) {
                        switch (table.substFormat) {
                            case 1:
                                trees.push(_6_1_1.default(table, allLookups, index));
                                break;
                            case 2:
                                trees.push(_6_2_1.default(table, allLookups, index));
                                break;
                            case 3:
                                trees.push(_6_3_1.default(table, allLookups, index));
                                break;
                        }
                    }
                    break;
                case 8:
                    for (const [index, table] of lookup.subtables.entries()) {
                        trees.push(_8_1_1.default(table, index));
                    }
                    break;
            }
            const tree = flatten_1.default(merge_1.default(trees));
            this._lookupTrees.push({
                tree,
                processForward: lookup.lookupType !== 8
            });
            for (const glyphId of Object.keys(tree)) {
                if (!this._glyphLookups[glyphId]) {
                    this._glyphLookups[glyphId] = [];
                }
                this._glyphLookups[glyphId].push(index);
            }
        }
    }
    findLigatures(text) {
        const cached = this._cache && this._cache.get(text);
        if (cached && !Array.isArray(cached)) {
            return cached;
        }
        const glyphIds = [];
        for (const char of text) {
            glyphIds.push(this._font.charToGlyphIndex(char));
        }
        // If there are no lookup groups, there's no point looking for
        // replacements. This gives us a minor performance boost for fonts with
        // no ligatures
        if (this._lookupTrees.length === 0) {
            return {
                inputGlyphs: glyphIds,
                outputGlyphs: glyphIds,
                contextRanges: []
            };
        }
        const result = this._findInternal(glyphIds.slice());
        const finalResult = {
            inputGlyphs: glyphIds,
            outputGlyphs: result.sequence,
            contextRanges: result.ranges
        };
        if (this._cache) {
            this._cache.set(text, finalResult);
        }
        return finalResult;
    }
    findLigatureRanges(text) {
        // Short circuit the process if there are no possible ligatures in the
        // font
        if (this._lookupTrees.length === 0) {
            return [];
        }
        const cached = this._cache && this._cache.get(text);
        if (cached) {
            return Array.isArray(cached) ? cached : cached.contextRanges;
        }
        const glyphIds = [];
        for (const char of text) {
            glyphIds.push(this._font.charToGlyphIndex(char));
        }
        const result = this._findInternal(glyphIds);
        if (this._cache) {
            this._cache.set(text, result.ranges);
        }
        return result.ranges;
    }
    _findInternal(sequence) {
        const ranges = [];
        let nextLookup = this._getNextLookup(sequence, 0);
        while (nextLookup.index !== null) {
            const lookup = this._lookupTrees[nextLookup.index];
            if (lookup.processForward) {
                let lastGlyphIndex = nextLookup.last;
                for (let i = nextLookup.first; i < lastGlyphIndex; i++) {
                    const result = walk_1.default(lookup.tree, sequence, i, i);
                    if (result) {
                        for (let j = 0; j < result.substitutions.length; j++) {
                            const sub = result.substitutions[j];
                            if (sub !== null) {
                                sequence[i + j] = sub;
                            }
                        }
                        mergeRange_1.default(ranges, result.contextRange[0] + i, result.contextRange[1] + i);
                        // Substitutions can end up extending the search range
                        if (i + result.length >= lastGlyphIndex) {
                            lastGlyphIndex = i + result.length + 1;
                        }
                        i += result.length - 1;
                    }
                }
            }
            else {
                // We don't need to do the lastGlyphIndex tracking here because
                // reverse processing isn't allowed to replace more than one
                // character at a time.
                for (let i = nextLookup.last - 1; i >= nextLookup.first; i--) {
                    const result = walk_1.default(lookup.tree, sequence, i, i);
                    if (result) {
                        for (let j = 0; j < result.substitutions.length; j++) {
                            const sub = result.substitutions[j];
                            if (sub !== null) {
                                sequence[i + j] = sub;
                            }
                        }
                        mergeRange_1.default(ranges, result.contextRange[0] + i, result.contextRange[1] + i);
                        i -= result.length - 1;
                    }
                }
            }
            nextLookup = this._getNextLookup(sequence, nextLookup.index + 1);
        }
        return { sequence, ranges };
    }
    /**
     * Returns the lookup and glyph range for the first lookup that might
     * contain a match.
     *
     * @param sequence Input glyph sequence
     * @param start The first input to try
     */
    _getNextLookup(sequence, start) {
        const result = {
            index: null,
            first: Infinity,
            last: -1
        };
        // Loop through each glyph and find the first valid lookup for it
        for (let i = 0; i < sequence.length; i++) {
            const lookups = this._glyphLookups[sequence[i]];
            if (!lookups) {
                continue;
            }
            for (let j = 0; j < lookups.length; j++) {
                const lookupIndex = lookups[j];
                if (lookupIndex >= start) {
                    // Update the lookup information if it's the one we're
                    // storing or earlier than it.
                    if (result.index === null || lookupIndex <= result.index) {
                        result.index = lookupIndex;
                        if (result.first > i) {
                            result.first = i;
                        }
                        result.last = i + 1;
                    }
                    break;
                }
            }
        }
        return result;
    }
}
/**
 * Load the font with the given name. The returned value can be used to find
 * ligatures for the font.
 *
 * @param name Font family name for the font to load
 */
async function load(name, options) {
    // We just grab the first font variant we find for now.
    // TODO: allow users to specify information to pick a specific variant
    const [fontInfo] = await Promise.resolve().then(() => require('font-finder')).then(fontFinder => fontFinder.listVariants(name));
    if (!fontInfo) {
        throw new Error(`Font ${name} not found`);
    }
    return loadFile(fontInfo.path, options);
}
exports.load = load;
/**
 * Load the font at the given file path. The returned value can be used to find
 * ligatures for the font.
 *
 * @param path Path to the file containing the font
 */
async function loadFile(path, options) {
    const font = await Promise.resolve().then(() => require('util')).then(util => util.promisify(opentype.load)(path));
    return new FontImpl(font, Object.assign({ cacheSize: 0 }, options));
}
exports.loadFile = loadFile;
/**
 * Load the font from it's binary data. The returned value can be used to find
 * ligatures for the font.
 *
 * @param buffer ArrayBuffer of the font to load
 */
function loadBuffer(buffer, options) {
    const font = opentype.parse(buffer);
    return new FontImpl(font, Object.assign({ cacheSize: 0 }, options));
}
exports.loadBuffer = loadBuffer;
//# sourceMappingURL=index.js.map