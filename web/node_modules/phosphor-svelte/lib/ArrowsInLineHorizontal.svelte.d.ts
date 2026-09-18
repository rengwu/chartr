import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsInLineHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsInLineHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsInLineHorizontal: Component<IconComponentProps, {}, "">;
type ArrowsInLineHorizontal = ReturnType<typeof ArrowsInLineHorizontal>;
export default ArrowsInLineHorizontal;
