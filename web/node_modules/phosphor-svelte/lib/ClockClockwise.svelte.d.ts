import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ClockClockwiseIcon` instead.
 *
 * @example
 * ```svelte
 * <ClockClockwise color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ClockClockwise: Component<IconComponentProps, {}, "">;
type ClockClockwise = ReturnType<typeof ClockClockwise>;
export default ClockClockwise;
