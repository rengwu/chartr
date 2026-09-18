import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsOutLineHorizontalIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsOutLineHorizontal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsOutLineHorizontal: Component<IconComponentProps, {}, "">;
type ArrowsOutLineHorizontal = ReturnType<typeof ArrowsOutLineHorizontal>;
export default ArrowsOutLineHorizontal;
