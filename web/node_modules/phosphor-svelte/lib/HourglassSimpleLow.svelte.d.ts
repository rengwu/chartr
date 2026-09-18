import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HourglassSimpleLowIcon` instead.
 *
 * @example
 * ```svelte
 * <HourglassSimpleLow color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassSimpleLow: Component<IconComponentProps, {}, "">;
type HourglassSimpleLow = ReturnType<typeof HourglassSimpleLow>;
export default HourglassSimpleLow;
