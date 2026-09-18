import { afterTick, attachRef, boxWith, } from "svelte-toolbelt";
import { Context, watch } from "runed";
import { boolToStr, boolToEmptyStrOrUndef, getDataOpenClosed, getDataTransitionAttrs, } from "../../internal/attrs.js";
import { kbd } from "../../internal/kbd.js";
import { createBitsAttrs } from "../../internal/attrs.js";
import { RovingFocusGroup } from "../../internal/roving-focus-group.js";
import { on } from "svelte/events";
import { PresenceManager } from "../../internal/presence-manager.svelte.js";
const accordionAttrs = createBitsAttrs({
    component: "accordion",
    parts: ["root", "trigger", "content", "item", "header"],
});
const AccordionRootContext = new Context("Accordion.Root");
const AccordionItemContext = new Context("Accordion.Item");
class AccordionBaseState {
    opts;
    rovingFocusGroup;
    attachment;
    constructor(opts) {
        this.opts = opts;
        this.rovingFocusGroup = new RovingFocusGroup({
            rootNode: this.opts.ref,
            candidateAttr: accordionAttrs.trigger,
            loop: this.opts.loop,
            orientation: this.opts.orientation,
        });
        this.attachment = attachRef(this.opts.ref);
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        "data-orientation": this.opts.orientation.current,
        "data-disabled": boolToEmptyStrOrUndef(this.opts.disabled.current),
        [accordionAttrs.root]: "",
        ...this.attachment,
    }));
}
class AccordionSingleState extends AccordionBaseState {
    opts;
    isMulti = false;
    constructor(opts) {
        super(opts);
        this.opts = opts;
        this.includesItem = this.includesItem.bind(this);
        this.toggleItem = this.toggleItem.bind(this);
    }
    includesItem(item) {
        return this.opts.value.current === item;
    }
    toggleItem(item) {
        this.opts.value.current = this.includesItem(item) ? "" : item;
    }
}
class AccordionMultiState extends AccordionBaseState {
    #value;
    isMulti = true;
    constructor(props) {
        super(props);
        this.#value = props.value;
        this.includesItem = this.includesItem.bind(this);
        this.toggleItem = this.toggleItem.bind(this);
    }
    includesItem(item) {
        return this.#value.current.includes(item);
    }
    toggleItem(item) {
        this.#value.current = this.includesItem(item)
            ? this.#value.current.filter((v) => v !== item)
            : [...this.#value.current, item];
    }
}
export class AccordionRootState {
    static create(props) {
        const { type, ...rest } = props;
        const rootState = type === "single"
            ? new AccordionSingleState(rest)
            : new AccordionMultiState(rest);
        return AccordionRootContext.set(rootState);
    }
}
export class AccordionItemState {
    static create(props) {
        return AccordionItemContext.set(new AccordionItemState({ ...props, rootState: AccordionRootContext.get() }));
    }
    opts;
    root;
    isActive = $derived.by(() => this.root.includesItem(this.opts.value.current));
    isDisabled = $derived.by(() => this.opts.disabled.current || this.root.opts.disabled.current);
    attachment;
    contentNode = $state(null);
    contentPresence;
    constructor(opts) {
        this.opts = opts;
        this.root = opts.rootState;
        this.updateValue = this.updateValue.bind(this);
        this.attachment = attachRef(this.opts.ref);
        this.contentPresence = new PresenceManager({
            ref: boxWith(() => this.contentNode),
            open: boxWith(() => this.isActive),
        });
    }
    updateValue() {
        this.root.toggleItem(this.opts.value.current);
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        "data-state": getDataOpenClosed(this.isActive),
        "data-disabled": boolToEmptyStrOrUndef(this.isDisabled),
        "data-orientation": this.root.opts.orientation.current,
        [accordionAttrs.item]: "",
        ...this.attachment,
    }));
}
export class AccordionTriggerState {
    opts;
    itemState;
    #root;
    #isDisabled = $derived.by(() => this.opts.disabled.current ||
        this.itemState.opts.disabled.current ||
        this.#root.opts.disabled.current);
    attachment;
    constructor(opts, itemState) {
        this.opts = opts;
        this.itemState = itemState;
        this.#root = itemState.root;
        this.onclick = this.onclick.bind(this);
        this.onkeydown = this.onkeydown.bind(this);
        this.attachment = attachRef(this.opts.ref);
    }
    static create(props) {
        return new AccordionTriggerState(props, AccordionItemContext.get());
    }
    onclick(e) {
        if (this.#isDisabled || e.button !== 0) {
            e.preventDefault();
            return;
        }
        this.itemState.updateValue();
    }
    onkeydown(e) {
        if (this.#isDisabled)
            return;
        if (e.key === kbd.SPACE || e.key === kbd.ENTER) {
            e.preventDefault();
            this.itemState.updateValue();
            return;
        }
        this.#root.rovingFocusGroup.handleKeydown(this.opts.ref.current, e);
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        disabled: this.#isDisabled,
        "aria-expanded": boolToStr(this.itemState.isActive),
        "aria-disabled": boolToStr(this.#isDisabled),
        "data-disabled": boolToEmptyStrOrUndef(this.#isDisabled),
        "data-state": getDataOpenClosed(this.itemState.isActive),
        "data-orientation": this.#root.opts.orientation.current,
        [accordionAttrs.trigger]: "",
        tabindex: this.opts.tabindex.current,
        onclick: this.onclick,
        onkeydown: this.onkeydown,
        ...this.attachment,
    }));
}
export class AccordionContentState {
    opts;
    item;
    attachment;
    #originalStyles = undefined;
    #isMountAnimationPrevented = false;
    #dimensions = $state({ width: 0, height: 0 });
    open = $derived.by(() => {
        if (this.opts.hiddenUntilFound.current)
            return this.item.isActive;
        return this.opts.forceMount.current || this.item.isActive;
    });
    constructor(opts, item) {
        this.opts = opts;
        this.item = item;
        this.#isMountAnimationPrevented = this.item.isActive;
        this.attachment = attachRef(this.opts.ref, (v) => (this.item.contentNode = v));
        // Prevent mount animations on initial render
        $effect(() => {
            const rAF = requestAnimationFrame(() => {
                this.#isMountAnimationPrevented = false;
            });
            return () => cancelAnimationFrame(rAF);
        });
        watch.pre([() => this.opts.ref.current, () => this.opts.hiddenUntilFound.current], ([node, hiddenUntilFound]) => {
            if (!node || !hiddenUntilFound)
                return;
            const handleBeforeMatch = () => {
                if (this.item.isActive)
                    return;
                // we need to defer opening until after browser completes search highlighting
                // otherwise the browser will immediately open the accordion
                // and the search highlighting will not be visible
                requestAnimationFrame(() => {
                    this.item.updateValue();
                });
            };
            return on(node, "beforematch", handleBeforeMatch);
        });
        // Handle dimension updates
        watch([() => this.open, () => this.opts.ref.current], this.#updateDimensions);
    }
    static create(props) {
        return new AccordionContentState(props, AccordionItemContext.get());
    }
    #updateDimensions = ([_, node]) => {
        if (!node)
            return;
        afterTick(() => {
            const element = this.opts.ref.current;
            if (!element)
                return;
            // store original styles on first run
            this.#originalStyles ??= {
                transitionDuration: element.style.transitionDuration,
                animationName: element.style.animationName,
            };
            // temporarily disable animations for measurement
            element.style.transitionDuration = "0s";
            element.style.animationName = "none";
            const rect = element.getBoundingClientRect();
            this.#dimensions = { width: rect.width, height: rect.height };
            // restore animations if not initial mount
            if (!this.#isMountAnimationPrevented && this.#originalStyles) {
                element.style.transitionDuration = this.#originalStyles.transitionDuration;
                element.style.animationName = this.#originalStyles.animationName;
            }
        });
    };
    get shouldRender() {
        return this.item.contentPresence.shouldRender;
    }
    snippetProps = $derived.by(() => ({ open: this.item.isActive }));
    props = $derived.by(() => ({
        id: this.opts.id.current,
        "data-state": getDataOpenClosed(this.item.isActive),
        ...getDataTransitionAttrs(this.item.contentPresence.transitionStatus),
        "data-disabled": boolToEmptyStrOrUndef(this.item.isDisabled),
        "data-orientation": this.item.root.opts.orientation.current,
        [accordionAttrs.content]: "",
        style: {
            "--bits-accordion-content-height": `${this.#dimensions.height}px`,
            "--bits-accordion-content-width": `${this.#dimensions.width}px`,
        },
        hidden: this.opts.hiddenUntilFound.current && !this.item.isActive
            ? "until-found"
            : undefined,
        ...(this.opts.hiddenUntilFound.current && !this.shouldRender
            ? {}
            : {
                hidden: this.opts.hiddenUntilFound.current
                    ? !this.shouldRender
                    : this.opts.forceMount.current
                        ? undefined
                        : !this.shouldRender,
            }),
        ...this.attachment,
    }));
}
export class AccordionHeaderState {
    opts;
    item;
    attachment;
    constructor(opts, item) {
        this.opts = opts;
        this.item = item;
        this.attachment = attachRef(this.opts.ref);
    }
    static create(props) {
        return new AccordionHeaderState(props, AccordionItemContext.get());
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        role: "heading",
        "aria-level": this.opts.level.current,
        "data-heading-level": this.opts.level.current,
        "data-state": getDataOpenClosed(this.item.isActive),
        "data-orientation": this.item.root.opts.orientation.current,
        [accordionAttrs.header]: "",
        ...this.attachment,
    }));
}
