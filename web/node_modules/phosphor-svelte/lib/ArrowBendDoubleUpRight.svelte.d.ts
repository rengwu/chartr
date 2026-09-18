import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendDoubleUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendDoubleUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendDoubleUpRight: Component<IconComponentProps, {}, "">;
type ArrowBendDoubleUpRight = ReturnType<typeof ArrowBendDoubleUpRight>;
export default ArrowBendDoubleUpRight;
