import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendRightDownIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendRightDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendRightDown: Component<IconComponentProps, {}, "">;
type ArrowBendRightDown = ReturnType<typeof ArrowBendRightDown>;
export default ArrowBendRightDown;
