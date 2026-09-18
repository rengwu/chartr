import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowElbowUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowElbowUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowElbowUpRight: Component<IconComponentProps, {}, "">;
type ArrowElbowUpRight = ReturnType<typeof ArrowElbowUpRight>;
export default ArrowElbowUpRight;
