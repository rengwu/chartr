import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SquareHalfBottomIcon` instead.
 *
 * @example
 * ```svelte
 * <SquareHalfBottom color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SquareHalfBottom: Component<IconComponentProps, {}, "">;
type SquareHalfBottom = ReturnType<typeof SquareHalfBottom>;
export default SquareHalfBottom;
