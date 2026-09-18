import { attachRef, boxWith, DOMContext, } from "svelte-toolbelt";
import { Context, watch } from "runed";
import { kbd } from "../../internal/kbd.js";
import { createBitsAttrs, boolToStr, getDataOpenClosed, getDataTransitionAttrs, } from "../../internal/attrs.js";
import { isElement, isTouch } from "../../internal/is.js";
import { PresenceManager } from "../../internal/presence-manager.svelte.js";
import { SafePolygon } from "../../internal/safe-polygon.svelte.js";
import { isTabbable } from "tabbable";
const popoverAttrs = createBitsAttrs({
    component: "popover",
    parts: ["root", "trigger", "content", "close", "overlay"],
});
const PopoverRootContext = new Context("Popover.Root");
export class PopoverRootState {
    static create(opts) {
        return PopoverRootContext.set(new PopoverRootState(opts));
    }
    opts;
    contentNode = $state(null);
    contentPresence;
    triggerNode = $state(null);
    overlayNode = $state(null);
    overlayPresence;
    // hover tracking state
    openedViaHover = $state(false);
    hasInteractedWithContent = $state(false);
    hoverCooldown = $state(false);
    closeDelay = $state(0);
    #closeTimeout = null;
    #domContext = null;
    constructor(opts) {
        this.opts = opts;
        this.contentPresence = new PresenceManager({
            ref: boxWith(() => this.contentNode),
            open: this.opts.open,
            onComplete: () => {
                this.opts.onOpenChangeComplete.current(this.opts.open.current);
            },
        });
        this.overlayPresence = new PresenceManager({
            ref: boxWith(() => this.overlayNode),
            open: this.opts.open,
        });
        watch(() => this.opts.open.current, (isOpen) => {
            if (!isOpen) {
                this.openedViaHover = false;
                this.hasInteractedWithContent = false;
                this.#clearCloseTimeout();
            }
        });
    }
    setDomContext(ctx) {
        this.#domContext = ctx;
    }
    #clearCloseTimeout() {
        if (this.#closeTimeout !== null && this.#domContext) {
            this.#domContext.clearTimeout(this.#closeTimeout);
            this.#closeTimeout = null;
        }
    }
    toggleOpen() {
        this.#clearCloseTimeout();
        this.opts.open.current = !this.opts.open.current;
    }
    handleClose() {
        this.#clearCloseTimeout();
        if (!this.opts.open.current)
            return;
        this.opts.open.current = false;
    }
    handleHoverOpen() {
        this.#clearCloseTimeout();
        if (this.opts.open.current)
            return;
        this.openedViaHover = true;
        this.opts.open.current = true;
    }
    handleHoverClose() {
        if (!this.opts.open.current)
            return;
        // only close if opened via hover and user hasn't interacted with content
        if (this.openedViaHover && !this.hasInteractedWithContent) {
            this.opts.open.current = false;
        }
    }
    handleDelayedHoverClose() {
        if (!this.opts.open.current)
            return;
        if (!this.openedViaHover || this.hasInteractedWithContent)
            return;
        this.#clearCloseTimeout();
        if (this.closeDelay <= 0) {
            this.opts.open.current = false;
        }
        else if (this.#domContext) {
            this.#closeTimeout = this.#domContext.setTimeout(() => {
                if (this.openedViaHover && !this.hasInteractedWithContent) {
                    this.opts.open.current = false;
                }
                this.#closeTimeout = null;
            }, this.closeDelay);
        }
    }
    cancelDelayedClose() {
        this.#clearCloseTimeout();
    }
    markInteraction() {
        this.hasInteractedWithContent = true;
        this.#clearCloseTimeout();
    }
}
export class PopoverTriggerState {
    static create(opts) {
        return new PopoverTriggerState(opts, PopoverRootContext.get());
    }
    opts;
    root;
    attachment;
    domContext;
    #openTimeout = null;
    #closeTimeout = null;
    #isHovering = $state(false);
    constructor(opts, root) {
        this.opts = opts;
        this.root = root;
        this.attachment = attachRef(this.opts.ref, (v) => (this.root.triggerNode = v));
        this.domContext = new DOMContext(opts.ref);
        this.root.setDomContext(this.domContext);
        this.onclick = this.onclick.bind(this);
        this.onkeydown = this.onkeydown.bind(this);
        this.onpointerenter = this.onpointerenter.bind(this);
        this.onpointerleave = this.onpointerleave.bind(this);
        watch(() => this.opts.closeDelay.current, (delay) => {
            this.root.closeDelay = delay;
        });
    }
    #clearOpenTimeout() {
        if (this.#openTimeout !== null) {
            this.domContext.clearTimeout(this.#openTimeout);
            this.#openTimeout = null;
        }
    }
    #clearCloseTimeout() {
        if (this.#closeTimeout !== null) {
            this.domContext.clearTimeout(this.#closeTimeout);
            this.#closeTimeout = null;
        }
    }
    #clearAllTimeouts() {
        this.#clearOpenTimeout();
        this.#clearCloseTimeout();
    }
    onpointerenter(e) {
        if (this.opts.disabled.current)
            return;
        if (!this.opts.openOnHover.current)
            return;
        if (isTouch(e))
            return;
        this.#isHovering = true;
        this.#clearCloseTimeout();
        this.root.cancelDelayedClose();
        if (this.root.opts.open.current || this.root.hoverCooldown)
            return;
        const delay = this.opts.openDelay.current;
        if (delay <= 0) {
            this.root.handleHoverOpen();
        }
        else {
            this.#openTimeout = this.domContext.setTimeout(() => {
                this.root.handleHoverOpen();
                this.#openTimeout = null;
            }, delay);
        }
    }
    onpointerleave(e) {
        if (this.opts.disabled.current)
            return;
        if (!this.opts.openOnHover.current)
            return;
        if (isTouch(e))
            return;
        this.#isHovering = false;
        this.#clearOpenTimeout();
        this.root.hoverCooldown = false;
        // let GraceArea handle the close - it will call handleHoverClose via onPointerExit
        // we just need to stop any pending open timer
    }
    onclick(e) {
        if (this.opts.disabled.current)
            return;
        if (e.button !== 0)
            return;
        this.#clearAllTimeouts();
        // if clicked while hovering and popover is open, convert to click-based open
        if (this.#isHovering && this.root.opts.open.current && this.root.openedViaHover) {
            this.root.openedViaHover = false;
            this.root.hasInteractedWithContent = true;
            return;
        }
        // if closing while hovering with openOnHover enabled, set cooldown to prevent
        // immediate re-open via hover
        if (this.#isHovering && this.opts.openOnHover.current && this.root.opts.open.current) {
            this.root.hoverCooldown = true;
        }
        // if clicking to open while in cooldown, reset cooldown (explicit open)
        if (this.root.hoverCooldown && !this.root.opts.open.current) {
            this.root.hoverCooldown = false;
        }
        this.root.toggleOpen();
    }
    onkeydown(e) {
        if (this.opts.disabled.current)
            return;
        if (!(e.key === kbd.ENTER || e.key === kbd.SPACE))
            return;
        e.preventDefault();
        this.#clearAllTimeouts();
        this.root.toggleOpen();
    }
    #getAriaControls() {
        if (this.root.opts.open.current && this.root.contentNode?.id) {
            return this.root.contentNode?.id;
        }
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        "aria-haspopup": "dialog",
        "aria-expanded": boolToStr(this.root.opts.open.current),
        "data-state": getDataOpenClosed(this.root.opts.open.current),
        "aria-controls": this.#getAriaControls(),
        [popoverAttrs.trigger]: "",
        disabled: this.opts.disabled.current,
        //
        onkeydown: this.onkeydown,
        onclick: this.onclick,
        onpointerenter: this.onpointerenter,
        onpointerleave: this.onpointerleave,
        ...this.attachment,
    }));
}
export class PopoverContentState {
    static create(opts) {
        return new PopoverContentState(opts, PopoverRootContext.get());
    }
    opts;
    root;
    attachment;
    constructor(opts, root) {
        this.opts = opts;
        this.root = root;
        this.attachment = attachRef(this.opts.ref, (v) => (this.root.contentNode = v));
        this.onpointerdown = this.onpointerdown.bind(this);
        this.onfocusin = this.onfocusin.bind(this);
        this.onpointerenter = this.onpointerenter.bind(this);
        this.onpointerleave = this.onpointerleave.bind(this);
        new SafePolygon({
            triggerNode: () => this.root.triggerNode,
            contentNode: () => this.root.contentNode,
            enabled: () => this.root.opts.open.current &&
                this.root.openedViaHover &&
                !this.root.hasInteractedWithContent,
            onPointerExit: () => {
                this.root.handleDelayedHoverClose();
            },
        });
    }
    onpointerdown(_) {
        this.root.markInteraction();
    }
    onfocusin(e) {
        const target = e.target;
        if (isElement(target) && isTabbable(target)) {
            this.root.markInteraction();
        }
    }
    onpointerenter(e) {
        if (isTouch(e))
            return;
        this.root.cancelDelayedClose();
    }
    onpointerleave(e) {
        if (isTouch(e))
            return;
        // handled by grace area
    }
    onInteractOutside = (e) => {
        this.opts.onInteractOutside.current(e);
        if (e.defaultPrevented)
            return;
        if (!isElement(e.target))
            return;
        const closestTrigger = e.target.closest(popoverAttrs.selector("trigger"));
        if (closestTrigger && closestTrigger === this.root.triggerNode)
            return;
        if (this.opts.customAnchor.current) {
            if (isElement(this.opts.customAnchor.current)) {
                if (this.opts.customAnchor.current.contains(e.target))
                    return;
            }
            else if (typeof this.opts.customAnchor.current === "string") {
                const el = document.querySelector(this.opts.customAnchor.current);
                if (el && el.contains(e.target))
                    return;
            }
        }
        this.root.handleClose();
    };
    onEscapeKeydown = (e) => {
        this.opts.onEscapeKeydown.current(e);
        if (e.defaultPrevented)
            return;
        this.root.handleClose();
    };
    get shouldRender() {
        return this.root.contentPresence.shouldRender;
    }
    get shouldTrapFocus() {
        if (this.root.openedViaHover && !this.root.hasInteractedWithContent)
            return false;
        return true;
    }
    snippetProps = $derived.by(() => ({ open: this.root.opts.open.current }));
    props = $derived.by(() => ({
        id: this.opts.id.current,
        tabindex: -1,
        "data-state": getDataOpenClosed(this.root.opts.open.current),
        ...getDataTransitionAttrs(this.root.contentPresence.transitionStatus),
        [popoverAttrs.content]: "",
        style: {
            pointerEvents: "auto",
            // CSS containment isolates style/layout/paint calculations from the rest of the page
            contain: "layout style",
        },
        onpointerdown: this.onpointerdown,
        onfocusin: this.onfocusin,
        onpointerenter: this.onpointerenter,
        onpointerleave: this.onpointerleave,
        ...this.attachment,
    }));
    popperProps = {
        onInteractOutside: this.onInteractOutside,
        onEscapeKeydown: this.onEscapeKeydown,
    };
}
export class PopoverCloseState {
    static create(opts) {
        return new PopoverCloseState(opts, PopoverRootContext.get());
    }
    opts;
    root;
    attachment;
    constructor(opts, root) {
        this.opts = opts;
        this.root = root;
        this.attachment = attachRef(this.opts.ref);
        this.onclick = this.onclick.bind(this);
        this.onkeydown = this.onkeydown.bind(this);
    }
    onclick(_) {
        this.root.handleClose();
    }
    onkeydown(e) {
        if (!(e.key === kbd.ENTER || e.key === kbd.SPACE))
            return;
        e.preventDefault();
        this.root.handleClose();
    }
    props = $derived.by(() => ({
        id: this.opts.id.current,
        onclick: this.onclick,
        onkeydown: this.onkeydown,
        type: "button",
        [popoverAttrs.close]: "",
        ...this.attachment,
    }));
}
export class PopoverOverlayState {
    static create(opts) {
        return new PopoverOverlayState(opts, PopoverRootContext.get());
    }
    opts;
    root;
    attachment;
    constructor(opts, root) {
        this.opts = opts;
        this.root = root;
        this.attachment = attachRef(this.opts.ref, (v) => (this.root.overlayNode = v));
    }
    get shouldRender() {
        return this.root.overlayPresence.shouldRender;
    }
    snippetProps = $derived.by(() => ({ open: this.root.opts.open.current }));
    props = $derived.by(() => ({
        id: this.opts.id.current,
        [popoverAttrs.overlay]: "",
        style: {
            pointerEvents: "auto",
        },
        "data-state": getDataOpenClosed(this.root.opts.open.current),
        ...getDataTransitionAttrs(this.root.overlayPresence.transitionStatus),
        ...this.attachment,
    }));
}
