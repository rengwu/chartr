import { type ReadableBoxedValues } from "svelte-toolbelt";
import type { TransitionState } from "./attrs.js";
interface PresenceManagerOpts extends ReadableBoxedValues<{
    open: boolean;
    ref: HTMLElement | null;
}> {
    onComplete?: () => void;
    enabled?: boolean;
    shouldSkipExitAnimation?: () => boolean;
}
export declare class PresenceManager {
    #private;
    constructor(opts: PresenceManagerOpts);
    get shouldRender(): boolean;
    get transitionStatus(): TransitionState;
}
export {};
