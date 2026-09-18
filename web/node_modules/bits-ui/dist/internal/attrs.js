export function boolToStr(condition) {
    return condition ? "true" : "false";
}
export function boolToStrTrueOrUndef(condition) {
    return condition ? "true" : undefined;
}
export function boolToEmptyStrOrUndef(condition) {
    return condition ? "" : undefined;
}
export function boolToTrueOrUndef(condition) {
    return condition ? true : undefined;
}
export function getDataOpenClosed(condition) {
    return condition ? "open" : "closed";
}
export function getDataChecked(condition) {
    return condition ? "checked" : "unchecked";
}
export function getDataTransitionAttrs(state) {
    if (state === "starting")
        return { "data-starting-style": "" };
    if (state === "ending")
        return { "data-ending-style": "" };
    return {};
}
export function getAriaChecked(checked, indeterminate) {
    if (indeterminate)
        return "mixed";
    return checked ? "true" : "false";
}
export class BitsAttrs {
    #variant;
    #prefix;
    attrs;
    constructor(config) {
        this.#variant = config.getVariant ? config.getVariant() : null;
        this.#prefix = this.#variant ? `data-${this.#variant}-` : `data-${config.component}-`;
        this.getAttr = this.getAttr.bind(this);
        this.selector = this.selector.bind(this);
        this.attrs = Object.fromEntries(config.parts.map((part) => [part, this.getAttr(part)]));
    }
    getAttr(part, variantOverride) {
        if (variantOverride)
            return `data-${variantOverride}-${part}`;
        return `${this.#prefix}${part}`;
    }
    selector(part, variantOverride) {
        return `[${this.getAttr(part, variantOverride)}]`;
    }
}
export function createBitsAttrs(config) {
    const bitsAttrs = new BitsAttrs(config);
    return {
        ...bitsAttrs.attrs,
        selector: bitsAttrs.selector,
        getAttr: bitsAttrs.getAttr,
    };
}
