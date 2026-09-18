import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HourglassSimpleMediumIcon` instead.
 *
 * @example
 * ```svelte
 * <HourglassSimpleMedium color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassSimpleMedium: Component<IconComponentProps, {}, "">;
type HourglassSimpleMedium = ReturnType<typeof HourglassSimpleMedium>;
export default HourglassSimpleMedium;
