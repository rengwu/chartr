import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CircleHalfIcon` instead.
 *
 * @example
 * ```svelte
 * <CircleHalf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CircleHalf: Component<IconComponentProps, {}, "">;
type CircleHalf = ReturnType<typeof CircleHalf>;
export default CircleHalf;
