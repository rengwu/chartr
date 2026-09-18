import type { Getter, MaybeGetter } from "../../internal/types.js";
export declare class Throttled<T> {
    #private;
    constructor(getter: Getter<T>, wait?: MaybeGetter<number>);
    get current(): T;
    cancel(): void;
    setImmediately(v: T): void;
}
