import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareNineIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareNine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareNine: Component<IconComponentProps, {}, "">;
type NumberSquareNine = ReturnType<typeof NumberSquareNine>;
export default NumberSquareNine;
