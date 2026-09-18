import { watch } from "runed";
import { onDestroyEffect } from "svelte-toolbelt";
import { AnimationsComplete } from "./animations-complete.js";
export class PresenceManager {
    #opts;
    #enabled;
    #afterAnimations;
    #shouldRender = $state(false);
    #transitionStatus = $state(undefined);
    #hasMounted = false;
    #transitionFrame = null;
    constructor(opts) {
        this.#opts = opts;
        this.#shouldRender = opts.open.current;
        this.#enabled = opts.enabled ?? true;
        this.#afterAnimations = new AnimationsComplete({
            ref: this.#opts.ref,
            afterTick: this.#opts.open,
        });
        onDestroyEffect(() => this.#clearTransitionFrame());
        watch(() => this.#opts.open.current, (isOpen) => {
            if (!this.#hasMounted) {
                this.#hasMounted = true;
                return;
            }
            this.#clearTransitionFrame();
            if (!isOpen && this.#opts.shouldSkipExitAnimation?.()) {
                this.#shouldRender = false;
                this.#transitionStatus = undefined;
                this.#opts.onComplete?.();
                return;
            }
            if (isOpen)
                this.#shouldRender = true;
            this.#transitionStatus = isOpen ? "starting" : "ending";
            if (isOpen) {
                this.#transitionFrame = window.requestAnimationFrame(() => {
                    this.#transitionFrame = null;
                    if (this.#opts.open.current) {
                        this.#transitionStatus = undefined;
                    }
                });
            }
            if (!this.#enabled) {
                if (!isOpen) {
                    this.#shouldRender = false;
                }
                this.#transitionStatus = undefined;
                this.#opts.onComplete?.();
                return;
            }
            this.#afterAnimations.run(() => {
                if (isOpen === this.#opts.open.current) {
                    if (!this.#opts.open.current) {
                        this.#shouldRender = false;
                    }
                    this.#transitionStatus = undefined;
                    this.#opts.onComplete?.();
                }
            });
        });
    }
    get shouldRender() {
        return this.#shouldRender;
    }
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
