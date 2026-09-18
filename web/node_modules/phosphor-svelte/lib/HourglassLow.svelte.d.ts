import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HourglassLowIcon` instead.
 *
 * @example
 * ```svelte
 * <HourglassLow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassLow: Component<IconComponentProps, {}, "">;
type HourglassLow = ReturnType<typeof HourglassLow>;
export default HourglassLow;
