import { untrack } from "svelte";
export function useThrottle(callback, interval = 250) {
    let lastCall = 0;
    let timeout = $state();
    let resolve = null;
    let reject = null;
    let promise = null;
    function reset() {
        timeout = undefined;
        promise = null;
        resolve = null;
        reject = null;
    }
    function throttled(...args) {
        return untrack(() => {
            const now = Date.now();
            const intervalValue = typeof interval === "function" ? interval() : interval;
            const nextAllowedTime = lastCall + intervalValue;
            if (!promise) {
                promise = new Promise((res, rej) => {
                    resolve = res;
                    reject = rej;
                });
            }
            if (now < nextAllowedTime) {
                if (!timeout) {
                    timeout = setTimeout(async () => {
                        try {
                            const result = await callback.apply(this, args);
                            resolve?.(result);
                        }
                        catch (error) {
                            reject?.(error);
                        }
                        finally {
                            clearTimeout(timeout);
                            reset();
                            lastCall = Date.now();
                        }
                    }, nextAllowedTime - now);
                }
                return promise;
            }
            if (timeout) {
                clearTimeout(timeout);
                timeout = undefined;
            }
            lastCall = now;
            try {
                const result = callback.apply(this, args);
                resolve?.(result);
            }
            catch (error) {
                reject?.(error);
            }
            finally {
                reset();
            }
            return promise;
        });
    }
    throttled.cancel = async () => {
        if (timeout) {
            if (timeout === undefined) {
                // Wait one event loop to see if something triggered the throttled function
                await new Promise((resolve) => setTimeout(resolve, 0));
                if (timeout === undefined)
                    return;
            }
            clearTimeout(timeout);
            reject?.("Cancelled");
            reset();
        }
    };
    Object.defineProperty(throttled, "pending", {
        enumerable: true,
        get() {
            return !!timeout;
        },
    });
    return throttled;
}
