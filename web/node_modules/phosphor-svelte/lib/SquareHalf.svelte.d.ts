import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SquareHalfIcon` instead.
 *
 * @example
 * ```svelte
 * <SquareHalf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SquareHalf: Component<IconComponentProps, {}, "">;
type SquareHalf = ReturnType<typeof SquareHalf>;
export default SquareHalf;
