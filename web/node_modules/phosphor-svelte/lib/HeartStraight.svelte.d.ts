import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeartStraightIcon` instead.
 *
 * @example
 * ```svelte
 * <HeartStraight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HeartStraight: Component<IconComponentProps, {}, "">;
type HeartStraight = ReturnType<typeof HeartStraight>;
export default HeartStraight;
