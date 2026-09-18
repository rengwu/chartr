import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsInCardinalIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsInCardinal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsInCardinal: Component<IconComponentProps, {}, "">;
type ArrowsInCardinal = ReturnType<typeof ArrowsInCardinal>;
export default ArrowsInCardinal;
