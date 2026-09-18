import { type Getter } from "svelte-toolbelt";
export interface SafePolygonOptions {
    enabled: Getter<boolean>;
    triggerNode: Getter<HTMLElement | null>;
    contentNode: Getter<HTMLElement | null>;
    onPointerExit: () => void;
    buffer?: number;
    transitIntentTimeout?: number;
    /** nodes that should not trigger a close when they become the relatedTarget on trigger leave (e.g. sibling triggers in singleton mode) */
    ignoredTargets?: Getter<HTMLElement[]>;
}
/**
 * Creates a safe polygon area that allows users to move their cursor between
 * the trigger and floating content without closing it.
 */
export declare class SafePolygon {
    #private;
    constructor(opts: SafePolygonOptions);
}
