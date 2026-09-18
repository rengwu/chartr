import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowElbowRightUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowElbowRightUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowElbowRightUp: Component<IconComponentProps, {}, "">;
type ArrowElbowRightUp = ReturnType<typeof ArrowElbowRightUp>;
export default ArrowElbowRightUp;
