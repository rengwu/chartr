import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendUpRight: Component<IconComponentProps, {}, "">;
type ArrowBendUpRight = ReturnType<typeof ArrowBendUpRight>;
export default ArrowBendUpRight;
