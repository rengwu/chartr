import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsClockwiseIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsClockwise color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsClockwise: Component<IconComponentProps, {}, "">;
type ArrowsClockwise = ReturnType<typeof ArrowsClockwise>;
export default ArrowsClockwise;
