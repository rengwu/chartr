import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeartStraightBreakIcon` instead.
 *
 * @example
 * ```svelte
 * <HeartStraightBreak color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HeartStraightBreak: Component<IconComponentProps, {}, "">;
type HeartStraightBreak = ReturnType<typeof HeartStraightBreak>;
export default HeartStraightBreak;
