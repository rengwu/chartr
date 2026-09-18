import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ClockIcon` instead.
 *
 * @example
 * ```svelte
 * <Clock color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Clock: Component<IconComponentProps, {}, "">;
type Clock = ReturnType<typeof Clock>;
export default Clock;
