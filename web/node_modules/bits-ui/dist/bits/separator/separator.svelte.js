import { attachRef } from "svelte-toolbelt";
import { createBitsAttrs, boolToStrTrueOrUndef } from "../../internal/attrs.js";
const separatorAttrs = createBitsAttrs({
    component: "separator",
    parts: ["root"],
});
export class SeparatorRootState {
    static create(opts) {
        return new SeparatorRootState(opts);
    }
    opts;
    attachment;
    constructor(opts) {
        this.opts = opts;
        this.attachment = attachRef(opts.ref);
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        role: this.opts.decorative.current ? "none" : "separator",
        "aria-orientation": this.opts.orientation.current,
        "aria-hidden": boolToStrTrueOrUndef(this.opts.decorative.current),
        "data-orientation": this.opts.orientation.current,
        [separatorAttrs.root]: "",
        ...this.attachment,
    }));
}
