import type { MaybeGetter } from "../../internal/types.js";
type UseThrottleReturn<Args extends unknown[], Return> = ((this: unknown, ...args: Args) => Promise<Return>) & {
    cancel: () => void;
    pending: boolean;
};
export declare function useThrottle<Args extends unknown[], Return>(callback: (...args: Args) => Return, interval?: MaybeGetter<number>): UseThrottleReturn<Args, Return>;
export {};
