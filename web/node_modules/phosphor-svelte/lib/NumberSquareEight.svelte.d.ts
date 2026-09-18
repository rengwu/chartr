import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareEightIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareEight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareEight: Component<IconComponentProps, {}, "">;
type NumberSquareEight = ReturnType<typeof NumberSquareEight>;
export default NumberSquareEight;
