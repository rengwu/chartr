import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowElbowRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowElbowRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowElbowRight: Component<IconComponentProps, {}, "">;
type ArrowElbowRight = ReturnType<typeof ArrowElbowRight>;
export default ArrowElbowRight;
