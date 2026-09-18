import { onDestroyEffect } from "svelte-toolbelt";
export class TimeoutFn {
    #interval;
    #cb;
    #timer = null;
    constructor(cb, interval) {
        this.#cb = cb;
        this.#interval = interval;
        this.stop = this.stop.bind(this);
        this.start = this.start.bind(this);
        onDestroyEffect(this.stop);
    }
    #clear() {
        if (this.#timer !== null) {
            window.clearTimeout(this.#timer);
            this.#timer = null;
        }
    }
    stop() {
        this.#clear();
    }
    start(...args) {
        this.#clear();
        this.#timer = window.setTimeout(() => {
            this.#timer = null;
            this.#cb(...args);
        }, this.#interval);
    }
}
