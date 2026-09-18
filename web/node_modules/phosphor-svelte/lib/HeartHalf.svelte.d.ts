import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HeartHalfIcon` instead.
 *
 * @example
 * ```svelte
 * <HeartHalf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HeartHalf: Component<IconComponentProps, {}, "">;
type HeartHalf = ReturnType<typeof HeartHalf>;
export default HeartHalf;
