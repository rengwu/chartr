import { afterTick, attachRef, boxWith, } from "svelte-toolbelt";
import { Context, watch } from "runed";
import { createBitsAttrs, boolToStr, boolToEmptyStrOrUndef, getDataOpenClosed, getDataTransitionAttrs, } from "../../internal/attrs.js";
import { kbd } from "../../internal/kbd.js";
import { on } from "svelte/events";
import { PresenceManager } from "../../internal/presence-manager.svelte.js";
const collapsibleAttrs = createBitsAttrs({
    component: "collapsible",
    parts: ["root", "content", "trigger"],
});
const CollapsibleRootContext = new Context("Collapsible.Root");
export class CollapsibleRootState {
    static create(opts) {
        return CollapsibleRootContext.set(new CollapsibleRootState(opts));
    }
    opts;
    attachment;
    contentNode = $state(null);
    contentPresence;
    contentId = $state(undefined);
    constructor(opts) {
        this.opts = opts;
        this.toggleOpen = this.toggleOpen.bind(this);
        this.attachment = attachRef(this.opts.ref);
        this.contentPresence = new PresenceManager({
            ref: boxWith(() => this.contentNode),
            open: this.opts.open,
            onComplete: () => {
                this.opts.onOpenChangeComplete.current(this.opts.open.current);
            },
        });
    }
    toggleOpen() {
        this.opts.open.current = !this.opts.open.current;
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        "data-state": getDataOpenClosed(this.opts.open.current),
        "data-disabled": boolToEmptyStrOrUndef(this.opts.disabled.current),
        [collapsibleAttrs.root]: "",
        ...this.attachment,
    }));
}
export class CollapsibleContentState {
    static create(opts) {
        return new CollapsibleContentState(opts, CollapsibleRootContext.get());
    }
    opts;
    root;
    attachment;
    present = $derived.by(() => {
        if (this.opts.hiddenUntilFound.current)
            return this.root.opts.open.current;
        return this.opts.forceMount.current || this.root.opts.open.current;
    });
    #originalStyles;
    #isMountAnimationPrevented = $state(false);
    #width = $state(0);
    #height = $state(0);
    constructor(opts, root) {
        this.opts = opts;
        this.root = root;
        this.#isMountAnimationPrevented = root.opts.open.current;
        this.root.contentId = this.opts.id.current;
        this.attachment = attachRef(this.opts.ref, (v) => (this.root.contentNode = v));
        watch.pre(() => this.opts.id.current, (id) => {
            this.root.contentId = id;
        });
        $effect.pre(() => {
            const rAF = requestAnimationFrame(() => {
                this.#isMountAnimationPrevented = false;
            });
            return () => {
                cancelAnimationFrame(rAF);
            };
        });
        watch.pre([() => this.opts.ref.current, () => this.opts.hiddenUntilFound.current], ([node, hiddenUntilFound]) => {
            if (!node || !hiddenUntilFound)
                return;
            const handleBeforeMatch = () => {
                if (this.root.opts.open.current)
                    return;
                // we need to defer opening until after browser completes search highlighting
                // otherwise the browser will immediately open the collapsible
                // and the search highlighting will not be visible
                requestAnimationFrame(() => {
                    this.root.opts.open.current = true;
                });
            };
            return on(node, "beforematch", handleBeforeMatch);
        });
        watch([() => this.opts.ref.current, () => this.present], ([node]) => {
            if (!node)
                return;
            afterTick(() => {
                if (!this.opts.ref.current)
                    return;
                // get the dimensions of the element
                this.#originalStyles = this.#originalStyles || {
                    transitionDuration: node.style.transitionDuration,
                    animationName: node.style.animationName,
                };
                // block any animations/transitions so the element renders at full dimensions
                node.style.transitionDuration = "0s";
                node.style.animationName = "none";
                const rect = node.getBoundingClientRect();
                this.#height = rect.height;
                this.#width = rect.width;
                // unblock any animations/transitions that were originally set if not the initial render
                if (!this.#isMountAnimationPrevented) {
                    const { animationName, transitionDuration } = this.#originalStyles;
                    node.style.transitionDuration = transitionDuration;
                    node.style.animationName = animationName;
                }
            });
        });
    }
    get shouldRender() {
        return this.root.contentPresence.shouldRender;
    }
    snippetProps = $derived.by(() => ({
        open: this.root.opts.open.current,
    }));
    props = $derived.by(() => ({
        id: this.opts.id.current,
        style: {
            "--bits-collapsible-content-height": this.#height
                ? `${this.#height}px`
                : undefined,
            "--bits-collapsible-content-width": this.#width
                ? `${this.#width}px`
                : undefined,
        },
        hidden: this.opts.hiddenUntilFound.current && !this.root.opts.open.current
            ? "until-found"
            : undefined,
        "data-state": getDataOpenClosed(this.root.opts.open.current),
        ...getDataTransitionAttrs(this.root.contentPresence.transitionStatus),
        "data-disabled": boolToEmptyStrOrUndef(this.root.opts.disabled.current),
        [collapsibleAttrs.content]: "",
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
export class CollapsibleTriggerState {
    static create(opts) {
        return new CollapsibleTriggerState(opts, CollapsibleRootContext.get());
    }
    opts;
    root;
    attachment;
    #isDisabled = $derived.by(() => this.opts.disabled.current || this.root.opts.disabled.current);
    constructor(opts, root) {
        this.opts = opts;
        this.root = root;
        this.attachment = attachRef(this.opts.ref);
        this.onclick = this.onclick.bind(this);
        this.onkeydown = this.onkeydown.bind(this);
    }
    onclick(e) {
        if (this.#isDisabled)
            return;
        if (e.button !== 0)
            return e.preventDefault();
        this.root.toggleOpen();
    }
    onkeydown(e) {
        if (this.#isDisabled)
            return;
        if (e.key === kbd.SPACE || e.key === kbd.ENTER) {
            e.preventDefault();
            this.root.toggleOpen();
        }
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        type: "button",
        disabled: this.#isDisabled,
        "aria-controls": this.root.contentId,
        "aria-expanded": boolToStr(this.root.opts.open.current),
        "data-state": getDataOpenClosed(this.root.opts.open.current),
        "data-disabled": boolToEmptyStrOrUndef(this.#isDisabled),
        [collapsibleAttrs.trigger]: "",
        //
        onclick: this.onclick,
        onkeydown: this.onkeydown,
        ...this.attachment,
    }));
}
