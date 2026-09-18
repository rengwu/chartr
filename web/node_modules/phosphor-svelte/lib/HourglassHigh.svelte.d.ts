import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HourglassHighIcon` instead.
 *
 * @example
 * ```svelte
 * <HourglassHigh color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassHigh: Component<IconComponentProps, {}, "">;
type HourglassHigh = ReturnType<typeof HourglassHigh>;
export default HourglassHigh;
