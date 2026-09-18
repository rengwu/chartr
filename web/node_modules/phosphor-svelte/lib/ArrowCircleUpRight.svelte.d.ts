import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowCircleUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowCircleUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowCircleUpRight: Component<IconComponentProps, {}, "">;
type ArrowCircleUpRight = ReturnType<typeof ArrowCircleUpRight>;
export default ArrowCircleUpRight;
