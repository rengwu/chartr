import type { AnyFn } from "./types.js";
export declare class TimeoutFn<T extends AnyFn> {
    #private;
    constructor(cb: T, interval: number);
    stop(): void;
    start(...args: Parameters<T> | []): void;
}
