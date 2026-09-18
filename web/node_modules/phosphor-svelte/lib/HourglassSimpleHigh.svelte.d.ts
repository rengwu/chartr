import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HourglassSimpleHighIcon` instead.
 *
 * @example
 * ```svelte
 * <HourglassSimpleHigh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassSimpleHigh: Component<IconComponentProps, {}, "">;
type HourglassSimpleHigh = ReturnType<typeof HourglassSimpleHigh>;
export default HourglassSimpleHigh;
