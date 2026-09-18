import type { MaybeGetter } from "../../internal/types.js";
export type UseIntervalOptions = {
    /**
     * Start the timer immediately
     *
     * @default true
     */
    immediate?: boolean;
    /**
     * Execute the callback immediately after calling `resume`
     *
     * @default false
     */
    immediateCallback?: boolean;
    /**
     * Callback to execute on every interval tick, receives the current counter value
     */
    callback?: (count: number) => void;
};
export type UseIntervalReturn = {
    /**
     * Pause the interval
     */
    pause: () => void;
    /**
     * Resume the interval
     */
    resume: () => void;
    /**
     * Reset the counter to 0
     */
    reset: () => void;
    /**
     * Whether the interval is currently active
     */
    readonly isActive: boolean;
    /**
     * The current counter value
     */
    readonly counter: number;
};
/**
 * Wrapper for `setInterval` with controls for pausing and resuming.
 *
 * @see https://runed.dev/docs/utilities/use-interval
 *
 * @param delay - The interval in milliseconds between executions
 * @param options - Configuration options
 * @returns Object with pause, resume, reset methods, counter and isActive state
 */
export declare function useInterval(delay: MaybeGetter<number>, options?: UseIntervalOptions): UseIntervalReturn;
