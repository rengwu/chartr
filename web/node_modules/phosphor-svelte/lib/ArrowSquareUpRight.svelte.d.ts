import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowSquareUpRightIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowSquareUpRight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowSquareUpRight: Component<IconComponentProps, {}, "">;
type ArrowSquareUpRight = ReturnType<typeof ArrowSquareUpRight>;
export default ArrowSquareUpRight;
