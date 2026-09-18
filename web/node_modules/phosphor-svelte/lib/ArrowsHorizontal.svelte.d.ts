import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsHorizontal: Component<IconComponentProps, {}, "">;
type ArrowsHorizontal = ReturnType<typeof ArrowsHorizontal>;
export default ArrowsHorizontal;
