import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StarHalfIcon` instead.
 *
 * @example
 * ```svelte
 * <StarHalf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const StarHalf: Component<IconComponentProps, {}, "">;
type StarHalf = ReturnType<typeof StarHalf>;
export default StarHalf;
