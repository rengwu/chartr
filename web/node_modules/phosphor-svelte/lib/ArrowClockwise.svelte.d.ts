import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowClockwiseIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowClockwise color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowClockwise: Component<IconComponentProps, {}, "">;
type ArrowClockwise = ReturnType<typeof ArrowClockwise>;
export default ArrowClockwise;
