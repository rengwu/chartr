import { boxFrom, boxWith, boxFlatten, toReadonlyBox, isBox, isWritableBox, BoxSymbol, isWritableSymbol } from "./box-extras.svelte.js";
export function box(initialValue) {
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
box.from = boxFrom;
box.with = boxWith;
box.flatten = boxFlatten;
box.readonly = toReadonlyBox;
box.isBox = isBox;
box.isWritableBox = isWritableBox;
