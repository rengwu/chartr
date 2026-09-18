import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsInIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsIn color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsIn: Component<IconComponentProps, {}, "">;
type ArrowsIn = ReturnType<typeof ArrowsIn>;
export default ArrowsIn;
