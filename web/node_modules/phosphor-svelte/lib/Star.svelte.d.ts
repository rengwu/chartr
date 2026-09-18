import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StarIcon` instead.
 *
 * @example
 * ```svelte
 * <Star color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Star: Component<IconComponentProps, {}, "">;
type Star = ReturnType<typeof Star>;
export default Star;
