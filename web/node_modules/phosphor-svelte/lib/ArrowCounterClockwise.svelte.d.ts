import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowCounterClockwiseIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowCounterClockwise color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowCounterClockwise: Component<IconComponentProps, {}, "">;
type ArrowCounterClockwise = ReturnType<typeof ArrowCounterClockwise>;
export default ArrowCounterClockwise;
