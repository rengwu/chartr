import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowElbowLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowElbowLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowElbowLeft: Component<IconComponentProps, {}, "">;
type ArrowElbowLeft = ReturnType<typeof ArrowElbowLeft>;
export default ArrowElbowLeft;
