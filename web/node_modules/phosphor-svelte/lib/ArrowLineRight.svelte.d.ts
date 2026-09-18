import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowLineRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowLineRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowLineRight: Component<IconComponentProps, {}, "">;
type ArrowLineRight = ReturnType<typeof ArrowLineRight>;
export default ArrowLineRight;
