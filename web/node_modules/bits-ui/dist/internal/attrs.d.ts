export declare function boolToStr(condition: boolean): "true" | "false";
export declare function boolToStrTrueOrUndef(condition: boolean): "true" | undefined;
export declare function boolToEmptyStrOrUndef(condition: boolean): "" | undefined;
export declare function boolToTrueOrUndef(condition: boolean): true | undefined;
export declare function getDataOpenClosed(condition: boolean): "open" | "closed";
export declare function getDataChecked(condition: boolean): "checked" | "unchecked";
export type TransitionState = "starting" | "ending" | "idle" | undefined;
export declare function getDataTransitionAttrs(state: TransitionState): {
    "data-starting-style"?: "";
    "data-ending-style"?: "";
};
export declare function getAriaChecked(checked: boolean, indeterminate: boolean): "true" | "false" | "mixed";
export type BitsAttrsConfig<T extends readonly string[]> = {
    component: string;
    parts: T;
    getVariant?: () => string | null;
};
export type CreateBitsAttrsReturn<T extends readonly string[]> = {
    [K in T[number]]: string;
} & {
    selector: (part: T[number]) => string;
    getAttr: (part: T[number], variant?: string) => string;
};
export declare class BitsAttrs<T extends readonly string[]> {
    #private;
    attrs: Record<T[number], string>;
    constructor(config: BitsAttrsConfig<T>);
    getAttr(part: T[number], variantOverride?: string): string;
    selector(part: T[number], variantOverride?: string): string;
}
export declare function createBitsAttrs<const T extends readonly string[]>(config: Omit<BitsAttrsConfig<T>, "parts"> & {
    parts: T;
}): CreateBitsAttrsReturn<T>;
