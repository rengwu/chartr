import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <HourglassSimpleHighIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HourglassSimpleHighIcon: Component<IconComponentProps, {}, "">;
type HourglassSimpleHighIcon = ReturnType<typeof HourglassSimpleHighIcon>;
export default HourglassSimpleHighIcon;
