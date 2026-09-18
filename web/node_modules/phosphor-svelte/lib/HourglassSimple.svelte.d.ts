import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HourglassSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <HourglassSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassSimple: Component<IconComponentProps, {}, "">;
type HourglassSimple = ReturnType<typeof HourglassSimple>;
export default HourglassSimple;
