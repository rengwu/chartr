import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareDownRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareDownRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareDownRight: Component<IconComponentProps, {}, "">;
type ArrowSquareDownRight = ReturnType<typeof ArrowSquareDownRight>;
export default ArrowSquareDownRight;
