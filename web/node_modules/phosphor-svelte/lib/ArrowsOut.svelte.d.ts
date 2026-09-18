import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsOutIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsOut color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsOut: Component<IconComponentProps, {}, "">;
type ArrowsOut = ReturnType<typeof ArrowsOut>;
export default ArrowsOut;
