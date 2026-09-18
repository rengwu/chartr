import { extract } from "../extract/index.js";
import { watch } from "../watch/watch.svelte.js";
/**
 * Wrapper for `setInterval` with controls for pausing and resuming.
 *
 * @see https://runed.dev/docs/utilities/use-interval
 *
 * @param delay - The interval in milliseconds between executions
 * @param options - Configuration options
 * @returns Object with pause, resume, reset methods, counter and isActive state
 */
export function useInterval(delay, options = {}) {
    const { immediate = true, immediateCallback = false, callback } = options;
    let intervalId = $state(null);
    let counter = $state(0);
    const delay$ = $derived(extract(delay));
    const isActive = $derived(intervalId !== null);
    function runCallback() {
        counter++;
        callback?.(counter);
    }
    function createInterval() {
        intervalId = setInterval(runCallback, delay$);
    }
    const pause = () => {
        if (intervalId === null)
            return;
        clearInterval(intervalId);
        intervalId = null;
    };
    const resume = () => {
        if (intervalId !== null)
            return;
        if (immediateCallback)
            runCallback();
        createInterval();
    };
    if (immediate) {
        resume();
    }
    // Sync interval's delay with the prop
    watch(() => delay$, () => {
        if (!isActive)
            return;
        pause();
        createInterval();
    });
    // Cleanup on disposal
    $effect(() => pause);
    return {
        pause,
        resume,
        reset: () => (counter = 0),
        get isActive() {
            return isActive;
        },
        get counter() {
            return counter;
        },
    };
}
