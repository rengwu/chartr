import { DOMContext, type WritableBoxedValues, type ReadableBoxedValues } from "svelte-toolbelt";
import type { OnChangeFn, RefAttachment, WithRefOpts } from "../../internal/types.js";
import type { FocusEventHandler, MouseEventHandler, PointerEventHandler } from "svelte/elements";
import { PresenceManager } from "../../internal/presence-manager.svelte.js";
export declare const tooltipAttrs: import("../../internal/attrs.js").CreateBitsAttrsReturn<readonly ["content", "trigger"]>;
type TooltipTriggerRecord = {
    id: string;
    node: HTMLElement | null;
    payload: unknown;
    disabled: boolean;
};
declare class TooltipTriggerRegistryState {
    #private;
    triggers: Map<string, TooltipTriggerRecord>;
    activeTriggerId: string | null;
    activeTriggerNode: HTMLElement | null;
    activePayload: {} | null;
    register: (record: TooltipTriggerRecord) => void;
    update: (record: TooltipTriggerRecord) => void;
    unregister: (id: string) => void;
    setActiveTrigger: (id: string | null) => void;
    get: (id: string) => TooltipTriggerRecord | undefined;
    has: (id: string) => boolean;
    getFirstTriggerId: () => string | null;
}
declare class TooltipTetherState {
    readonly registry: TooltipTriggerRegistryState;
    root: TooltipRootState | null;
}
export declare class TooltipTether<Payload = never> {
    #private;
    get state(): TooltipTetherState;
    open(triggerId: string): void;
    close(): void;
    get isOpen(): boolean;
}
export declare function createTooltipTether<Payload = never>(): TooltipTether<Payload>;
interface TooltipProviderStateOpts extends ReadableBoxedValues<{
    delayDuration: number;
    disableHoverableContent: boolean;
    disableCloseOnTriggerClick: boolean;
    disabled: boolean;
    ignoreNonKeyboardFocus: boolean;
    skipDelayDuration: number;
}> {
}
export declare class TooltipProviderState {
    #private;
    static create(opts: TooltipProviderStateOpts): TooltipProviderState;
    readonly opts: TooltipProviderStateOpts;
    isOpenDelayed: boolean;
    isPointerInTransit: import("svelte-toolbelt").WritableBox<boolean>;
    constructor(opts: TooltipProviderStateOpts);
    onOpen: (tooltip: TooltipRootState) => void;
    onClose: (tooltip: TooltipRootState) => void;
    isTooltipOpen: (tooltip: TooltipRootState) => boolean;
}
interface TooltipRootStateOpts extends ReadableBoxedValues<{
    delayDuration: number | undefined;
    disableHoverableContent: boolean | undefined;
    disableCloseOnTriggerClick: boolean | undefined;
    disabled: boolean | undefined;
    ignoreNonKeyboardFocus: boolean | undefined;
    onOpenChangeComplete: OnChangeFn<boolean>;
    tether: TooltipTether<unknown> | undefined;
}>, WritableBoxedValues<{
    open: boolean;
    triggerId: string | null;
}> {
}
export declare class TooltipRootState {
    #private;
    static create(opts: TooltipRootStateOpts): TooltipRootState;
    readonly opts: TooltipRootStateOpts;
    readonly provider: TooltipProviderState;
    readonly delayDuration: number;
    readonly disableHoverableContent: boolean;
    readonly disableCloseOnTriggerClick: boolean;
    readonly disabled: boolean;
    readonly ignoreNonKeyboardFocus: boolean;
    readonly registry: TooltipTriggerRegistryState;
    readonly tether: TooltipTetherState | null;
    contentNode: HTMLElement | null;
    contentPresence: PresenceManager;
    readonly stateAttr: string;
    constructor(opts: TooltipRootStateOpts, provider: TooltipProviderState);
    handleOpen: () => void;
    handleClose: () => void;
    onTriggerEnter: (triggerId: string) => void;
    onTriggerLeave: () => void;
    ensureActiveTrigger: () => void;
    setActiveTrigger: (triggerId: string | null) => void;
    registerTrigger: (trigger: TooltipTriggerRecord) => void;
    updateTrigger: (trigger: TooltipTriggerRecord) => void;
    unregisterTrigger: (id: string) => void;
    isActiveTrigger: (triggerId: string) => boolean;
    get triggerNode(): HTMLElement | null;
    get activePayload(): {} | null;
    get activeTriggerId(): string | null;
}
interface TooltipTriggerStateOpts extends WithRefOpts, ReadableBoxedValues<{
    disabled: boolean;
    tabindex: number;
    payload: unknown;
    tether: TooltipTether<unknown> | undefined;
}> {
}
export declare class TooltipTriggerState {
    #private;
    static create(opts: TooltipTriggerStateOpts): TooltipTriggerState;
    readonly opts: TooltipTriggerStateOpts;
    readonly root: TooltipRootState | null;
    readonly tether: TooltipTetherState | null;
    readonly attachment: RefAttachment;
    domContext: DOMContext;
    constructor(opts: TooltipTriggerStateOpts, root: TooltipRootState | null, tether: TooltipTetherState | null);
    handlePointerUp: () => void;
    readonly props: {
        readonly id: string;
        readonly "aria-describedby": string | undefined;
        readonly "data-state": "closed" | "delayed-open" | "instant-open" | undefined;
        readonly "data-disabled": "" | undefined;
        readonly "data-delay-duration": `${number}`;
        readonly tabindex: number | undefined;
        readonly disabled: boolean;
        readonly onpointerup: PointerEventHandler<HTMLElement>;
        readonly onpointerdown: PointerEventHandler<HTMLElement>;
        readonly onpointerenter: PointerEventHandler<HTMLElement>;
        readonly onpointermove: PointerEventHandler<HTMLElement>;
        readonly onpointerleave: PointerEventHandler<HTMLElement>;
        readonly onfocus: FocusEventHandler<HTMLElement>;
        readonly onblur: FocusEventHandler<HTMLElement>;
        readonly onclick: MouseEventHandler<HTMLElement>;
    };
}
interface TooltipContentStateOpts extends WithRefOpts, ReadableBoxedValues<{
    onInteractOutside: (e: PointerEvent) => void;
    onEscapeKeydown: (e: KeyboardEvent) => void;
}> {
}
export declare class TooltipContentState {
    static create(opts: TooltipContentStateOpts): TooltipContentState;
    readonly opts: TooltipContentStateOpts;
    readonly root: TooltipRootState;
    readonly attachment: RefAttachment;
    constructor(opts: TooltipContentStateOpts, root: TooltipRootState);
    onInteractOutside: (e: PointerEvent) => void;
    onEscapeKeydown: (e: KeyboardEvent) => void;
    onOpenAutoFocus: (e: Event) => void;
    onCloseAutoFocus: (e: Event) => void;
    get shouldRender(): boolean;
    readonly snippetProps: {
        open: boolean;
    };
    readonly props: {
        readonly style: {
            readonly outline: "none";
        };
        readonly "data-starting-style"?: "";
        readonly "data-ending-style"?: "";
        readonly id: string;
        readonly "data-state": "closed" | "delayed-open" | "instant-open";
        readonly "data-disabled": "" | undefined;
    };
    readonly popperProps: {
        onInteractOutside: (e: PointerEvent) => void;
        onEscapeKeydown: (e: KeyboardEvent) => void;
        onOpenAutoFocus: (e: Event) => void;
        onCloseAutoFocus: (e: Event) => void;
    };
}
export {};
