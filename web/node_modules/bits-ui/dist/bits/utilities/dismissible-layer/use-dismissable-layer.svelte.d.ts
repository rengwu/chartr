import { type ReadableBox, type WritableBox, type ReadableBoxedValues } from "svelte-toolbelt";
import type { DismissibleLayerImplProps, InteractOutsideBehaviorType } from "./types.js";
interface DismissibleLayerStateOpts extends ReadableBoxedValues<Required<Omit<DismissibleLayerImplProps, "children" | "ref">>> {
    ref: WritableBox<HTMLElement | null>;
}
export declare class DismissibleLayerState {
    #private;
    static create(opts: DismissibleLayerStateOpts): DismissibleLayerState;
    readonly opts: DismissibleLayerStateOpts;
    constructor(opts: DismissibleLayerStateOpts);
    props: {
        onfocuscapture: () => void;
        onblurcapture: () => void;
    };
}
export declare function getTopMostDismissableLayer(layersArr?: [DismissibleLayerState, ReadableBox<InteractOutsideBehaviorType>][]): [DismissibleLayerState, ReadableBox<InteractOutsideBehaviorType>] | undefined;
export type FocusOutsideEvent = CustomEvent<{
    originalEvent: FocusEvent;
}>;
export {};
