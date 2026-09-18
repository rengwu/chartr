import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowLineUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowLineUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowLineUpRight: Component<IconComponentProps, {}, "">;
type ArrowLineUpRight = ReturnType<typeof ArrowLineUpRight>;
export default ArrowLineUpRight;
