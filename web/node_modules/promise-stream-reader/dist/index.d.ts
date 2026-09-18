/// <reference types="node" />
import { Writable } from 'stream';
export interface PromiseStream extends Writable {
    /**
     * The current read position in the buffer. This number will only increment
     * upon the completion of the data requested by `read()` or `skip()`.
     */
    offset: number;
    /**
     * Read the specified number of bytes from the stream. Returns a promise
     * that resolves with a Buffer containing the requested data once available
     * or throws an error if the stream is closed or ends before all of the data
     * could be retrieved.
     *
     * @param size The number of bytes to read
     */
    read(size: number): Promise<Buffer>;
    /**
     * Skip the specified number of bytes in the stream. Returns a promise
     * that resolves once the requested number of bytes have been read and
     * skipped or throws an error if the stream is closed or ends before all of
     * the data could be skipped.
     *
     * @param size The number of bytes to skip
     */
    skip(size: number): Promise<void>;
}
declare function create(): PromiseStream;
export default create;
