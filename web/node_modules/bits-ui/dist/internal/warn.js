import { DEV } from "esm-env";
let set;
if (DEV) {
    set = new Set();
}
export function warn(...messages) {
    if (!DEV)
        return;
    const msg = messages.join(" ");
    if (set.has(msg))
        return;
    set.add(msg);
    // oxlint-disable-next-line no-console
    console.warn(`[Bits UI]: ${msg}`);
}
