import { onDestroyEffect } from "svelte-toolbelt";
import { watch } from "runed";
import { AnimationsComplete } from "../../../internal/animations-complete.js";
export class Presence {
    opts;
    present;
    #afterAnimations;
    #isPresent = $state(false);
    #hasMounted = false;
    #transitionStatus = $state(undefined);
    #transitionFrame = null;
    constructor(opts) {
        this.opts = opts;
        this.present = this.opts.open;
        this.#isPresent = opts.open.current;
        this.#afterAnimations = new AnimationsComplete({
            ref: this.opts.ref,
            afterTick: this.opts.open,
        });
        onDestroyEffect(() => this.#clearTransitionFrame());
        watch(() => this.present.current, (isOpen) => {
            if (!this.#hasMounted) {
                this.#hasMounted = true;
                return;
            }
            this.#clearTransitionFrame();
            if (isOpen) {
                this.#isPresent = true;
            }
            this.#transitionStatus = isOpen ? "starting" : "ending";
            if (isOpen) {
                this.#transitionFrame = window.requestAnimationFrame(() => {
                    this.#transitionFrame = null;
                    if (this.present.current) {
                        this.#transitionStatus = undefined;
                    }
                });
            }
            this.#afterAnimations.run(() => {
                if (isOpen !== this.present.current)
                    return;
                if (!isOpen) {
                    this.#isPresent = false;
                }
                this.#transitionStatus = undefined;
            });
        });
    }
    isPresent = $derived.by(() => {
        return this.#isPresent;
    });
    get transitionStatus() {
        return this.#transitionStatus;
    }
    #clearTransitionFrame() {
        if (this.#transitionFrame === null)
            return;
        window.cancelAnimationFrame(this.#transitionFrame);
        this.#transitionFrame = null;
    }
}
