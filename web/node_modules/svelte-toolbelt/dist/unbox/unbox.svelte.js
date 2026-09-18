import { isBox } from "../box/box-extras.svelte.js";
import { isFunction } from "../utils/is.js";
export function unbox(value) {
    if (isBox(value))
        return value.current;
    if (isFunction(value)) {
        const getter = value;
        return getter();
    }
    return value;
}
