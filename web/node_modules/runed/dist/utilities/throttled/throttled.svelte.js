import { useThrottle } from "../use-throttle/use-throttle.svelte.js";
import { watch } from "../watch/watch.svelte.js";
import { noop } from "../../internal/utils/function.js";
export class Throttled {
    #current = $state();
    #throttleFn;
    constructor(getter, wait = 250) {
        this.#current = getter(); // Immediately set the initial value
        this.#throttleFn = useThrottle(() => {
            this.#current = getter();
        }, wait);
        watch(getter, () => {
            this.#throttleFn()?.catch(noop);
        });
    }
    get current() {
        return this.#current;
    }
    cancel() {
        this.#throttleFn.cancel();
    }
    setImmediately(v) {
        this.cancel();
        this.#current = v;
    }
}
