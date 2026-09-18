import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShieldStarIcon` instead.
 *
 * @example
 * ```svelte
 * <ShieldStar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShieldStar: Component<IconComponentProps, {}, "">;
type ShieldStar = ReturnType<typeof ShieldStar>;
export default ShieldStar;
