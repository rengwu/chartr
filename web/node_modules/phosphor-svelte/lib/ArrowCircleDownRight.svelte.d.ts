import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowCircleDownRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowCircleDownRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowCircleDownRight: Component<IconComponentProps, {}, "">;
type ArrowCircleDownRight = ReturnType<typeof ArrowCircleDownRight>;
export default ArrowCircleDownRight;
