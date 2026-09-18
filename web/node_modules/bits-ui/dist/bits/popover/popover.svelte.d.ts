import { type ReadableBoxedValues, type WritableBoxedValues, DOMContext } from "svelte-toolbelt";
import type { BitsFocusEvent, BitsKeyboardEvent, BitsMouseEvent, BitsPointerEvent, OnChangeFn, RefAttachment, WithRefOpts } from "../../internal/types.js";
import type { Measurable } from "../../internal/floating-svelte/types.js";
import { PresenceManager } from "../../internal/presence-manager.svelte.js";
interface PopoverRootStateOpts extends WritableBoxedValues<{
    open: boolean;
}>, ReadableBoxedValues<{
    onOpenChangeComplete: OnChangeFn<boolean>;
}> {
}
export declare class PopoverRootState {
    #private;
    static create(opts: PopoverRootStateOpts): PopoverRootState;
    readonly opts: PopoverRootStateOpts;
    contentNode: HTMLElement | null;
    contentPresence: PresenceManager;
    triggerNode: HTMLElement | null;
    overlayNode: HTMLElement | null;
    overlayPresence: PresenceManager;
    openedViaHover: boolean;
    hasInteractedWithContent: boolean;
    hoverCooldown: boolean;
    closeDelay: number;
    constructor(opts: PopoverRootStateOpts);
    setDomContext(ctx: DOMContext): void;
    toggleOpen(): void;
    handleClose(): void;
    handleHoverOpen(): void;
    handleHoverClose(): void;
    handleDelayedHoverClose(): void;
    cancelDelayedClose(): void;
    markInteraction(): void;
}
interface PopoverTriggerStateOpts extends WithRefOpts, ReadableBoxedValues<{
    disabled: boolean;
    openOnHover: boolean;
    openDelay: number;
    closeDelay: number;
}> {
}
export declare class PopoverTriggerState {
    #private;
    static create(opts: PopoverTriggerStateOpts): PopoverTriggerState;
    readonly opts: PopoverTriggerStateOpts;
    readonly root: PopoverRootState;
    readonly attachment: RefAttachment;
    readonly domContext: DOMContext;
    constructor(opts: PopoverTriggerStateOpts, root: PopoverRootState);
    onpointerenter(e: BitsPointerEvent): void;
    onpointerleave(e: BitsPointerEvent): void;
    onclick(e: BitsMouseEvent): void;
    onkeydown(e: BitsKeyboardEvent): void;
    readonly props: {
        readonly id: string;
        readonly "aria-haspopup": "dialog";
        readonly "aria-expanded": "true" | "false";
        readonly "data-state": "open" | "closed";
        readonly "aria-controls": string | undefined;
        readonly disabled: boolean;
        readonly onkeydown: (e: BitsKeyboardEvent) => void;
        readonly onclick: (e: BitsMouseEvent) => void;
        readonly onpointerenter: (e: BitsPointerEvent) => void;
        readonly onpointerleave: (e: BitsPointerEvent) => void;
    };
}
interface PopoverContentStateOpts extends WithRefOpts, ReadableBoxedValues<{
    onInteractOutside: (e: PointerEvent) => void;
    onEscapeKeydown: (e: KeyboardEvent) => void;
    customAnchor: string | HTMLElement | null | Measurable;
}> {
}
export declare class PopoverContentState {
    static create(opts: PopoverContentStateOpts): PopoverContentState;
    readonly opts: PopoverContentStateOpts;
    readonly root: PopoverRootState;
    readonly attachment: RefAttachment;
    constructor(opts: PopoverContentStateOpts, root: PopoverRootState);
    onpointerdown(_: BitsPointerEvent): void;
    onfocusin(e: BitsFocusEvent): void;
    onpointerenter(e: BitsPointerEvent): void;
    onpointerleave(e: BitsPointerEvent): void;
    onInteractOutside: (e: PointerEvent) => void;
    onEscapeKeydown: (e: KeyboardEvent) => void;
    get shouldRender(): boolean;
    get shouldTrapFocus(): boolean;
    readonly snippetProps: {
        open: boolean;
    };
    readonly props: {
        readonly style: {
            readonly pointerEvents: "auto";
            readonly contain: "layout style";
        };
        readonly onpointerdown: (_: BitsPointerEvent) => void;
        readonly onfocusin: (e: BitsFocusEvent) => void;
        readonly onpointerenter: (e: BitsPointerEvent) => void;
        readonly onpointerleave: (e: BitsPointerEvent) => void;
        readonly "data-starting-style"?: "";
        readonly "data-ending-style"?: "";
        readonly id: string;
        readonly tabindex: -1;
        readonly "data-state": "open" | "closed";
    };
    readonly popperProps: {
        onInteractOutside: (e: PointerEvent) => void;
        onEscapeKeydown: (e: KeyboardEvent) => void;
    };
}
interface PopoverCloseStateOpts extends WithRefOpts {
}
export declare class PopoverCloseState {
    static create(opts: PopoverCloseStateOpts): PopoverCloseState;
    readonly opts: PopoverCloseStateOpts;
    readonly root: PopoverRootState;
    readonly attachment: RefAttachment;
    constructor(opts: PopoverCloseStateOpts, root: PopoverRootState);
    onclick(_: BitsPointerEvent): void;
    onkeydown(e: BitsKeyboardEvent): void;
    readonly props: {
        readonly id: string;
        readonly onclick: (_: BitsPointerEvent) => void;
        readonly onkeydown: (e: BitsKeyboardEvent) => void;
        readonly type: "button";
    };
}
interface PopoverOverlayStateOpts extends WithRefOpts {
}
export declare class PopoverOverlayState {
    static create(opts: PopoverOverlayStateOpts): PopoverOverlayState;
    readonly opts: PopoverOverlayStateOpts;
    readonly root: PopoverRootState;
    readonly attachment: RefAttachment;
    constructor(opts: PopoverOverlayStateOpts, root: PopoverRootState);
    get shouldRender(): boolean;
    readonly snippetProps: {
        open: boolean;
    };
    readonly props: {
        readonly "data-starting-style"?: "";
        readonly "data-ending-style"?: "";
        readonly id: string;
        readonly style: {
            readonly pointerEvents: "auto";
        };
        readonly "data-state": "open" | "closed";
    };
}
export {};
