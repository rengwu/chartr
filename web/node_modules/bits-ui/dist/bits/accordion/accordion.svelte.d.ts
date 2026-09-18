import { type Box, type ReadableBoxedValues, type WritableBoxedValues } from "svelte-toolbelt";
import type { BitsKeyboardEvent, BitsMouseEvent, RefAttachment, WithRefOpts } from "../../internal/types.js";
import type { Orientation } from "../../shared/index.js";
import { RovingFocusGroup } from "../../internal/roving-focus-group.js";
import { PresenceManager } from "../../internal/presence-manager.svelte.js";
interface AccordionBaseStateOpts extends WithRefOpts, ReadableBoxedValues<{
    disabled: boolean;
    orientation: Orientation;
    loop: boolean;
}> {
}
interface AccordionSingleStateOpts extends AccordionBaseStateOpts, WritableBoxedValues<{
    value: string;
}> {
}
interface AccordionMultiStateOpts extends AccordionBaseStateOpts, WritableBoxedValues<{
    value: string[];
}> {
}
type AccordionRoot = AccordionSingleState | AccordionMultiState;
interface AccordionItemStateOpts extends WithRefOpts, ReadableBoxedValues<{
    value: string;
    disabled: boolean;
}> {
    rootState: AccordionRoot;
}
interface AccordionTriggerStateOpts extends WithRefOpts, ReadableBoxedValues<{
    disabled: boolean | null | undefined;
    tabindex: number;
}> {
}
interface AccordionContentStateOpts extends WithRefOpts, ReadableBoxedValues<{
    forceMount: boolean;
    hiddenUntilFound: boolean;
}> {
}
interface AccordionHeaderStateOpts extends WithRefOpts, ReadableBoxedValues<{
    level: 1 | 2 | 3 | 4 | 5 | 6;
}> {
}
interface AccordionRootStateOpts extends WithRefOpts, ReadableBoxedValues<{
    disabled: boolean;
    orientation: Orientation;
    loop: boolean;
}> {
    type: "single" | "multiple";
    value: Box<string> | Box<string[]>;
}
declare abstract class AccordionBaseState {
    readonly opts: AccordionBaseStateOpts;
    readonly rovingFocusGroup: RovingFocusGroup;
    abstract readonly isMulti: boolean;
    readonly attachment: RefAttachment;
    constructor(opts: AccordionBaseStateOpts);
    abstract includesItem(item: string): boolean;
    abstract toggleItem(item: string): void;
    readonly props: {
        readonly id: string;
        readonly "data-orientation": Orientation;
        readonly "data-disabled": "" | undefined;
    };
}
declare class AccordionSingleState extends AccordionBaseState {
    readonly opts: AccordionSingleStateOpts;
    readonly isMulti: false;
    constructor(opts: AccordionSingleStateOpts);
    includesItem(item: string): boolean;
    toggleItem(item: string): void;
}
declare class AccordionMultiState extends AccordionBaseState {
    #private;
    readonly isMulti: true;
    constructor(props: AccordionMultiStateOpts);
    includesItem(item: string): boolean;
    toggleItem(item: string): void;
}
export declare class AccordionRootState {
    static create(props: AccordionRootStateOpts): AccordionRoot;
}
export declare class AccordionItemState {
    static create(props: Omit<AccordionItemStateOpts, "rootState">): AccordionItemState;
    readonly opts: AccordionItemStateOpts;
    readonly root: AccordionRoot;
    readonly isActive: boolean;
    readonly isDisabled: boolean;
    readonly attachment: RefAttachment;
    contentNode: HTMLElement | null;
    contentPresence: PresenceManager;
    constructor(opts: AccordionItemStateOpts);
    updateValue(): void;
    readonly props: {
        readonly id: string;
        readonly "data-state": "open" | "closed";
        readonly "data-disabled": "" | undefined;
        readonly "data-orientation": Orientation;
    };
}
export declare class AccordionTriggerState {
    #private;
    readonly opts: AccordionTriggerStateOpts;
    readonly itemState: AccordionItemState;
    readonly attachment: RefAttachment;
    constructor(opts: AccordionTriggerStateOpts, itemState: AccordionItemState);
    static create(props: AccordionTriggerStateOpts): AccordionTriggerState;
    onclick(e: BitsMouseEvent): void;
    onkeydown(e: BitsKeyboardEvent): void;
    readonly props: {
        readonly id: string;
        readonly disabled: boolean;
        readonly "aria-expanded": "true" | "false";
        readonly "aria-disabled": "true" | "false";
        readonly "data-disabled": "" | undefined;
        readonly "data-state": "open" | "closed";
        readonly "data-orientation": Orientation;
        readonly tabindex: number;
        readonly onclick: (e: BitsMouseEvent) => void;
        readonly onkeydown: (e: BitsKeyboardEvent) => void;
    };
}
export declare class AccordionContentState {
    #private;
    readonly opts: AccordionContentStateOpts;
    readonly item: AccordionItemState;
    readonly attachment: RefAttachment;
    readonly open: boolean;
    constructor(opts: AccordionContentStateOpts, item: AccordionItemState);
    static create(props: AccordionContentStateOpts): AccordionContentState;
    get shouldRender(): boolean;
    readonly snippetProps: {
        open: boolean;
    };
    readonly props: {
        readonly hidden: boolean | "until-found" | undefined;
        readonly "data-disabled": "" | undefined;
        readonly "data-orientation": Orientation;
        readonly style: {
            readonly "--bits-accordion-content-height": `${number}px`;
            readonly "--bits-accordion-content-width": `${number}px`;
        };
        readonly "data-starting-style"?: "";
        readonly "data-ending-style"?: "";
        readonly id: string;
        readonly "data-state": "open" | "closed";
    };
}
export declare class AccordionHeaderState {
    readonly opts: AccordionHeaderStateOpts;
    readonly item: AccordionItemState;
    readonly attachment: RefAttachment;
    constructor(opts: AccordionHeaderStateOpts, item: AccordionItemState);
    static create(props: AccordionHeaderStateOpts): AccordionHeaderState;
    readonly props: {
        readonly id: string;
        readonly role: "heading";
        readonly "aria-level": 1 | 2 | 3 | 4 | 5 | 6;
        readonly "data-heading-level": 1 | 2 | 3 | 4 | 5 | 6;
        readonly "data-state": "open" | "closed";
        readonly "data-orientation": Orientation;
    };
}
export {};
