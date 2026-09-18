import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowElbowRightDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowElbowRightDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowElbowRightDown: Component<IconComponentProps, {}, "">;
type ArrowElbowRightDown = ReturnType<typeof ArrowElbowRightDown>;
export default ArrowElbowRightDown;
