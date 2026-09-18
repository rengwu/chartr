import { boxFrom, boxWith, boxFlatten, toReadonlyBox, type WritableBox } from "./box-extras.svelte.js";
/**
 * Creates a writable box.
 *
 * @returns A box with a `current` property which can be set to a new value.
 * Useful to pass state to other functions.
 *
 * @see {@link https://runed.dev/docs/functions/box}
 */
export declare function box<T>(): WritableBox<T | undefined>;
/**
 * Creates a writable box with an initial value.
 *
 * @param initialValue The initial value of the box.
 * @returns A box with a `current` property which can be set to a new value.
 * Useful to pass state to other functions.
 *
 * @see {@link https://runed.dev/docs/functions/box}
 */
export declare function box<T>(initialValue: T): WritableBox<T>;
export declare namespace box {
    export var from: typeof boxFrom;
    var _a: typeof boxWith;
    export var flatten: typeof boxFlatten;
    export var readonly: typeof toReadonlyBox;
    export var isBox: typeof import("./box-extras.svelte.js").isBox;
    export var isWritableBox: typeof import("./box-extras.svelte.js").isWritableBox;
    export { _a as with };
}
