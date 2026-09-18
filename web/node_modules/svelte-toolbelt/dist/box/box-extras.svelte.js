import { isFunction, isObject } from "../utils/is.js";
export const BoxSymbol = Symbol("box");
export const isWritableSymbol = Symbol("is-writable");
function boxWith(getter, setter) {
    const derived = $derived.by(getter);
    if (setter) {
        return {
            [BoxSymbol]: true,
            [isWritableSymbol]: true,
            get current() {
                return derived;
            },
            set current(v) {
                setter(v);
            }
        };
    }
    return {
        [BoxSymbol]: true,
        get current() {
            return getter();
        }
    };
}
/**
 * @returns Whether the value is a Box
 *
 * @see {@link https://runed.dev/docs/functions/box}
 */
function isBox(value) {
    return isObject(value) && BoxSymbol in value;
}
/**
 * @returns Whether the value is a WritableBox
 *
 * @see {@link https://runed.dev/docs/functions/box}
 */
function isWritableBox(value) {
    return isBox(value) && isWritableSymbol in value;
}
function boxFrom(value) {
    if (isBox(value))
        return value;
    if (isFunction(value))
        return boxWith(value);
    return simpleBox(value);
}
/**
 * Function that gets an object of boxes, and returns an object of reactive values
 *
 * @example
 * const count = box(0)
 * const flat = box.flatten({ count, double: box.with(() => count.current) })
 * // type of flat is { count: number, readonly double: number }
 *
 * @see {@link https://runed.dev/docs/functions/box}
 */
function boxFlatten(boxes) {
    return Object.entries(boxes).reduce((acc, [key, b]) => {
        if (!isBox(b)) {
            return Object.assign(acc, { [key]: b });
        }
        if (isWritableBox(b)) {
            Object.defineProperty(acc, key, {
                get() {
                    return b.current;
                },
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                set(v) {
                    b.current = v;
                }
            });
        }
        else {
            Object.defineProperty(acc, key, {
                get() {
                    return b.current;
                }
            });
        }
        return acc;
    }, {});
}
/**
 * Function that converts a box to a readonly box.
 *
 * @example
 * const count = box(0) // WritableBox<number>
 * const countReadonly = box.readonly(count) // ReadableBox<number>
 *
 * @see {@link https://runed.dev/docs/functions/box}
 */
function toReadonlyBox(b) {
    if (!isWritableBox(b))
        return b;
    return {
        [BoxSymbol]: true,
        get current() {
            return b.current;
        }
    };
}
function simpleBox(initialValue) {
    let current = $state(initialValue);
    return {
        [BoxSymbol]: true,
        [isWritableSymbol]: true,
        get current() {
            return current;
        },
        set current(v) {
            current = v;
        }
    };
}
export { boxWith, isBox, isWritableBox, boxFrom, boxFlatten, toReadonlyBox, simpleBox };
