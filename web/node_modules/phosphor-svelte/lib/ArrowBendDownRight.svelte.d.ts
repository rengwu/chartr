import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowBendDownRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowBendDownRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowBendDownRight: Component<IconComponentProps, {}, "">;
type ArrowBendDownRight = ReturnType<typeof ArrowBendDownRight>;
export default ArrowBendDownRight;
