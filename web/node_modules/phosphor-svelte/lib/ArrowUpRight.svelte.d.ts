import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowUpRight: Component<IconComponentProps, {}, "">;
type ArrowUpRight = ReturnType<typeof ArrowUpRight>;
export default ArrowUpRight;
