"use strict";
var __rest = (this && this.__rest) || function (s, e) {
    var t = {};
    for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p) && e.indexOf(p) < 0)
        t[p] = s[p];
    if (s != null && typeof Object.getOwnPropertySymbols === "function")
        for (var i = 0, p = Object.getOwnPropertySymbols(s); i < p.length; i++) if (e.indexOf(p[i]) < 0)
            t[p[i]] = s[p[i]];
    return t;
};
Object.defineProperty(exports, "__esModule", { value: true });
const get_system_fonts_1 = require("get-system-fonts");
const parse_1 = require("./parse");
const extract = require("./extract");
var extract_1 = require("./extract");
exports.Type = extract_1.Type;
exports.Style = extract_1.Style;
/**
 * Retrieve metadata for all fonts installed on the system.
 *
 * @param options Options to configure font retrieval
 */
async function list(options) {
    const opts = Object.assign({ concurrency: 4, language: 'en', onFontError: null }, options);
    // TODO: support woff, woff2, ttc
    const files = await get_system_fonts_1.default({ extensions: ['ttf', 'otf'] });
    // Process each font in parallel, swallowing any errors found along the way.
    const results = await parallelize(async (file) => {
        try {
            const fontData = await parse_1.default(file);
            return getMetadata(file, fontData, opts.language);
        }
        catch (e) {
            if (opts.onFontError) {
                opts.onFontError(file, e);
            }
        }
    }, files, opts.concurrency);
    // Group the fonts by their font family
    const fonts = {};
    for (let _a of results.filter(font => font)) {
        const { name } = _a, font = __rest(_a, ["name"]);
        if (!fonts[name]) {
            fonts[name] = [];
        }
        fonts[name].push(font);
    }
    return fonts;
}
exports.list = list;
/**
 * Lists all variants found for the provided font family. If no variants are
 * found, an empty array is returned.
 *
 * @param name The name of the font family to retrieve
 * @param options Options to configure font retrieval
 */
async function listVariants(name, options) {
    const fonts = await list(options);
    return fonts[name] || [];
}
exports.listVariants = listVariants;
/**
 * Gets metadata for a single font file, returning metadata for the first
 * font variant found in the file. If there is an error extracting the font, an
 * error is thrown (unlike in list, where the font is simply ignored).
 *
 * @param path Absolute path to the file to retrieve
 * @param options Options to configure font retrieval
 */
async function get(path, options) {
    const opts = Object.assign({ language: 'en' }, options);
    const fontData = await parse_1.default(path);
    return getMetadata(path, fontData, opts.language);
}
exports.get = get;
/**
 * Extracts font metadata, given the font information.
 *
 * @param path Absolute path to the font file
 * @param fontData Table data for the font
 * @param language Language to use when resolving names
 */
function getMetadata(path, fontData, language) {
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
async function parallelize(operation, data, concurrency) {
    const results = [];
    let index = 0;
    const wrapper = async (i) => {
        results.push(await operation(data[i]));
        if (index < data.length) {
            await wrapper(index++);
        }
    };
    const promises = [];
    for (; index < data.length && index < concurrency; index++) {
        promises.push(wrapper(index));
    }
    await Promise.all(promises);
    return results;
}
//# sourceMappingURL=index.js.map