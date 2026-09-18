import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsCounterClockwiseIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsCounterClockwise color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsCounterClockwise: Component<IconComponentProps, {}, "">;
type ArrowsCounterClockwise = ReturnType<typeof ArrowsCounterClockwise>;
export default ArrowsCounterClockwise;
