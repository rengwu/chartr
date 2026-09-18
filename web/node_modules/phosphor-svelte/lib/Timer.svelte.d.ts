import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `TimerIcon` instead.
 *
 * @example
 * ```svelte
 * <Timer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Timer: Component<IconComponentProps, {}, "">;
type Timer = ReturnType<typeof Timer>;
export default Timer;
