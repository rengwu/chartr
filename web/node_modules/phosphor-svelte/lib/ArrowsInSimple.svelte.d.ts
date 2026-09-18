import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowsInSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowsInSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowsInSimple: Component<IconComponentProps, {}, "">;
type ArrowsInSimple = ReturnType<typeof ArrowsInSimple>;
export default ArrowsInSimple;
