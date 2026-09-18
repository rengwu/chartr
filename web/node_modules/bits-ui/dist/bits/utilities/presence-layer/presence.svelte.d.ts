import { type ReadableBox, type ReadableBoxedValues } from "svelte-toolbelt";
import type { TransitionState } from "../../../internal/attrs.js";
export interface PresenceOptions extends ReadableBoxedValues<{
    open: boolean;
    ref: HTMLElement | null;
}> {
}
export declare class Presence {
    #private;
    readonly opts: PresenceOptions;
    present: ReadableBox<boolean>;
    constructor(opts: PresenceOptions);
    isPresent: boolean;
    get transitionStatus(): TransitionState;
}
