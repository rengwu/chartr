import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareZeroIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareZero color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareZero: Component<IconComponentProps, {}, "">;
type NumberSquareZero = ReturnType<typeof NumberSquareZero>;
export default NumberSquareZero;
