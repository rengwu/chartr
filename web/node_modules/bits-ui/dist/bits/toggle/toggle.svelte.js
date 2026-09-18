import { attachRef } from "svelte-toolbelt";
import { createBitsAttrs, boolToStr, boolToEmptyStrOrUndef, boolToTrueOrUndef, } from "../../internal/attrs.js";
export const toggleAttrs = createBitsAttrs({
    component: "toggle",
    parts: ["root"],
});
export class ToggleRootState {
    static create(opts) {
        return new ToggleRootState(opts);
    }
    opts;
    attachment;
    constructor(opts) {
        this.opts = opts;
        this.attachment = attachRef(this.opts.ref);
        this.onclick = this.onclick.bind(this);
    }
    onclick(_) {
        if (this.opts.disabled.current)
            return;
        this.opts.pressed.current = !this.opts.pressed.current;
    }
    snippetProps = $derived.by(() => ({
        pressed: this.opts.pressed.current,
    }));
    props = $derived.by(() => ({
        [toggleAttrs.root]: "",
        id: this.opts.id.current,
        "data-disabled": boolToEmptyStrOrUndef(this.opts.disabled.current),
        "aria-pressed": boolToStr(this.opts.pressed.current),
        "data-state": getToggleDataState(this.opts.pressed.current),
        disabled: boolToTrueOrUndef(this.opts.disabled.current),
        onclick: this.onclick,
        ...this.attachment,
    }));
}
export function getToggleDataState(condition) {
    return condition ? "on" : "off";
}
