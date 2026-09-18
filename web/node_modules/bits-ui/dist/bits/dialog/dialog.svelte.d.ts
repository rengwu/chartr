import { type ReadableBoxedValues, type WritableBoxedValues } from "svelte-toolbelt";
import type { BitsKeyboardEvent, BitsMouseEvent, OnChangeFn, RefAttachment, WithRefOpts } from "../../internal/types.js";
import { PresenceManager } from "../../internal/presence-manager.svelte.js";
type DialogVariant = "alert-dialog" | "dialog";
declare const dialogAttrs: import("../../internal/attrs.js").CreateBitsAttrsReturn<readonly ["content", "trigger", "overlay", "title", "description", "close", "cancel", "action"]>;
interface DialogRootStateOpts extends WritableBoxedValues<{
    open: boolean;
}>, ReadableBoxedValues<{
    variant: DialogVariant;
    onOpenChangeComplete: OnChangeFn<boolean>;
}> {
}
export declare class DialogRootState {
    static create(opts: DialogRootStateOpts): DialogRootState;
    readonly opts: DialogRootStateOpts;
    triggerNode: HTMLElement | null;
    contentNode: HTMLElement | null;
    overlayNode: HTMLElement | null;
    descriptionNode: HTMLElement | null;
    contentId: string | undefined;
    titleId: string | undefined;
    triggerId: string | undefined;
    descriptionId: string | undefined;
    cancelNode: HTMLElement | null;
    nestedOpenCount: number;
    readonly depth: number;
    readonly parent: DialogRootState | null;
    contentPresence: PresenceManager;
    overlayPresence: PresenceManager;
    constructor(opts: DialogRootStateOpts, parent: DialogRootState | null);
    handleOpen(): void;
    handleClose(): void;
    getBitsAttr: typeof dialogAttrs.getAttr;
    incrementNested(): void;
    decrementNested(): void;
    readonly sharedProps: {
        readonly "data-state": "open" | "closed";
    };
}
interface DialogTriggerStateOpts extends WithRefOpts, ReadableBoxedValues<{
    disabled: boolean;
}> {
}
export declare class DialogTriggerState {
    static create(opts: DialogTriggerStateOpts): DialogTriggerState;
    readonly opts: DialogTriggerStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: DialogTriggerStateOpts, root: DialogRootState);
    onclick(e: BitsMouseEvent): void;
    onkeydown(e: BitsKeyboardEvent): void;
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly id: string;
        readonly "aria-haspopup": "dialog";
        readonly "aria-expanded": "true" | "false";
        readonly "aria-controls": string | undefined;
        readonly onkeydown: (e: BitsKeyboardEvent) => void;
        readonly onclick: (e: BitsMouseEvent) => void;
        readonly disabled: true | undefined;
    };
}
interface DialogCloseStateOpts extends WithRefOpts, ReadableBoxedValues<{
    variant: "action" | "cancel" | "close";
    disabled: boolean;
}> {
}
export declare class DialogCloseState {
    static create(opts: DialogCloseStateOpts): DialogCloseState;
    readonly opts: DialogCloseStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: DialogCloseStateOpts, root: DialogRootState);
    onclick(e: BitsMouseEvent): void;
    onkeydown(e: BitsKeyboardEvent): void;
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly id: string;
        readonly onclick: (e: BitsMouseEvent) => void;
        readonly onkeydown: (e: BitsKeyboardEvent) => void;
        readonly disabled: true | undefined;
        readonly tabindex: 0;
    };
}
interface DialogActionStateOpts extends WithRefOpts {
}
export declare class DialogActionState {
    static create(opts: DialogActionStateOpts): DialogActionState;
    readonly opts: DialogActionStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: DialogActionStateOpts, root: DialogRootState);
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly id: string;
    };
}
interface DialogTitleStateOpts extends WithRefOpts, ReadableBoxedValues<{
    level: 1 | 2 | 3 | 4 | 5 | 6;
}> {
}
export declare class DialogTitleState {
    static create(opts: DialogTitleStateOpts): DialogTitleState;
    readonly opts: DialogTitleStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: DialogTitleStateOpts, root: DialogRootState);
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly id: string;
        readonly role: "heading";
        readonly "aria-level": 1 | 2 | 3 | 4 | 5 | 6;
    };
}
interface DialogDescriptionStateOpts extends WithRefOpts {
}
export declare class DialogDescriptionState {
    static create(opts: DialogDescriptionStateOpts): DialogDescriptionState;
    readonly opts: DialogDescriptionStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: DialogDescriptionStateOpts, root: DialogRootState);
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly id: string;
    };
}
interface DialogContentStateOpts extends WithRefOpts {
}
export declare class DialogContentState {
    static create(opts: DialogContentStateOpts): DialogContentState;
    readonly opts: DialogContentStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: DialogContentStateOpts, root: DialogRootState);
    readonly snippetProps: {
        open: boolean;
    };
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly "data-starting-style"?: "";
        readonly "data-ending-style"?: "";
        readonly id: string;
        readonly role: "dialog" | "alertdialog";
        readonly "aria-modal": "true";
        readonly "aria-describedby": string | undefined;
        readonly "aria-labelledby": string | undefined;
        readonly style: {
            readonly pointerEvents: "auto";
            readonly outline: "none" | undefined;
            readonly "--bits-dialog-depth": number;
            readonly "--bits-dialog-nested-count": number;
            readonly contain: "layout style";
        };
        readonly tabindex: -1 | undefined;
        readonly "data-nested-open": "" | undefined;
        readonly "data-nested": "" | undefined;
    };
    get shouldRender(): boolean;
}
interface DialogOverlayStateOpts extends WithRefOpts {
}
export declare class DialogOverlayState {
    static create(opts: DialogOverlayStateOpts): DialogOverlayState;
    readonly opts: DialogOverlayStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: DialogOverlayStateOpts, root: DialogRootState);
    readonly snippetProps: {
        open: boolean;
    };
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly "data-starting-style"?: "";
        readonly "data-ending-style"?: "";
        readonly id: string;
        readonly style: {
            readonly pointerEvents: "auto";
            readonly "--bits-dialog-depth": number;
            readonly "--bits-dialog-nested-count": number;
        };
        readonly "data-nested-open": "" | undefined;
        readonly "data-nested": "" | undefined;
    };
    get shouldRender(): boolean;
}
interface AlertDialogCancelStateOpts extends WithRefOpts, ReadableBoxedValues<{
    disabled: boolean;
}> {
}
export declare class AlertDialogCancelState {
    static create(opts: AlertDialogCancelStateOpts): AlertDialogCancelState;
    readonly opts: AlertDialogCancelStateOpts;
    readonly root: DialogRootState;
    readonly attachment: RefAttachment;
    constructor(opts: AlertDialogCancelStateOpts, root: DialogRootState);
    onclick(e: BitsMouseEvent): void;
    onkeydown(e: BitsKeyboardEvent): void;
    readonly props: {
        readonly "data-state": "open" | "closed";
        readonly id: string;
        readonly onclick: (e: BitsMouseEvent) => void;
        readonly onkeydown: (e: BitsKeyboardEvent) => void;
        readonly tabindex: 0;
    };
}
export {};
