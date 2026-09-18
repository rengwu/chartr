"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const fs = require("fs");
const promise_stream_reader_1 = require("promise-stream-reader");
const name_1 = require("./tables/name");
const ltag_1 = require("./tables/ltag");
const os2_1 = require("./tables/os2");
const head_1 = require("./tables/head");
const post_1 = require("./tables/post");
var SignatureType;
(function (SignatureType) {
    SignatureType[SignatureType["TrueType"] = 0] = "TrueType";
    SignatureType[SignatureType["CFF"] = 1] = "CFF";
    SignatureType[SignatureType["Woff"] = 2] = "Woff";
})(SignatureType || (SignatureType = {}));
const tableInfo = {
    name: {
        tag: Buffer.from('name'),
        parse: name_1.default
    },
    ltag: {
        tag: Buffer.from('ltag'),
        parse: ltag_1.default
    },
    os2: {
        tag: Buffer.from('OS/2'),
        parse: os2_1.default
    },
    head: {
        tag: Buffer.from('head'),
        parse: head_1.default
    },
    post: {
        tag: Buffer.from('post'),
        parse: post_1.default
    }
};
/**
 * Loads the bare minimum information needed to retrieve the metadata that we
 * want, streaming the data from the file until we've found everything we need.
 *
 * @param filePath Absolute path to the font to load
 */
async function parseFont(filePath) {
    return new Promise((resolve, reject) => {
        (async () => {
            const pStream = promise_stream_reader_1.default();
            const stream = fs.createReadStream(filePath);
            // Track the stream state so we don't try to destroy a closed socket
            let streamFinished = false;
            const markFinished = () => { streamFinished = true; };
            stream.once('close', markFinished);
            stream.once('end', markFinished);
            stream.once('error', e => {
                streamFinished = true;
                reject(e);
            });
            stream.pipe(pStream);
            try {
                const signature = parseTag(await pStream.read(4));
                switch (signature) {
                    case SignatureType.TrueType:
                    case SignatureType.CFF:
                        const numTables = (await pStream.read(2)).readUInt16BE(0);
                        // Skip the rest of the offset table
                        await pStream.skip(6);
                        // Get the table metadata
                        const tableMeta = await findTableRecords(pStream, numTables);
                        // Order the tables based on location in the file. We
                        // want to look for earlier tables first
                        const orderedTables = Object.entries(tableMeta)
                            .sort((a, b) => a[1].offset - b[1].offset);
                        // Get the buffer representing each of the tables
                        const tableData = {};
                        for (const [name, meta] of orderedTables) {
                            // Skip the data between the end of the previous
                            // table and the start of this one
                            await pStream.skip(meta.offset - pStream.offset);
                            tableData[name] = await pStream.read(meta.length);
                        }
                        // The ltag table is usually not present, but parse it
                        // first if it is because we need it for the name table.
                        let ltag = [];
                        if (tableData.ltag) {
                            ltag = tableInfo.ltag.parse(tableData.ltag);
                        }
                        // If any of the necessary font tables are missing,
                        // throw
                        if (!tableData.name) {
                            throw new Error(`missing required OpenType table 'name' in font file: ${filePath}`);
                        }
                        // Parse and return the tables we need
                        return {
                            names: tableInfo.name.parse(tableData.name, ltag),
                            os2: tableData.os2 && tableInfo.os2.parse(tableData.os2),
                            head: tableData.head && tableInfo.head.parse(tableData.head),
                            post: tableData.post && tableInfo.post.parse(tableData.post)
                        };
                    case SignatureType.Woff:
                    default:
                        throw new Error('provided font type is not supported yet');
                }
            }
            finally {
                // Clean up our state so that the file stream doesn't leak
                stream.unpipe(pStream);
                if (!streamFinished) {
                    stream.destroy();
                    pStream.destroy();
                }
            }
        })().then(resolve, reject);
    });
}
exports.default = parseFont;
const signatures = {
    one: Buffer.from([0x00, 0x01, 0x00, 0x00]),
    otto: Buffer.from('OTTO'),
    true: Buffer.from('true'),
    typ1: Buffer.from('typ1'),
    woff: Buffer.from('wOFF')
};
/**
 * Parses a tag buffer, returning the type of font contained within.
 *
 * @param tag 4-byte buffer to parse for the tag
 */
function parseTag(tag) {
    if (tag.equals(signatures.one) ||
        tag.equals(signatures.true) ||
        tag.equals(signatures.typ1)) {
        return SignatureType.TrueType;
    }
    else if (tag.equals(signatures.otto)) {
        return SignatureType.CFF;
    }
    else if (tag.equals(signatures.woff)) {
        return SignatureType.Woff;
    }
    else {
        throw new Error(`Unsupported signature type: ${tag}`);
    }
}
/**
 * Parse the table record list for the specific tables that we care about.
 *
 * @param stream Promise stream positioned at the table record list
 * @param numTables The number of tables in the table record list
 */
async function findTableRecords(stream, numTables) {
    const tableMeta = {};
    for (let i = 0; i < numTables; i++) {
        const tag = await stream.read(4);
        const data = await stream.read(12);
        for (const [name, table] of Object.entries(tableInfo)) {
            if (tag.equals(table.tag)) {
                tableMeta[name] = {
                    offset: data.readUInt32BE(4),
                    length: data.readUInt32BE(8)
                };
                if (tableMeta.name && tableMeta.ltag && tableMeta.os2) {
                    return tableMeta;
                }
            }
        }
    }
    return tableMeta;
}
//# sourceMappingURL=parse.js.map