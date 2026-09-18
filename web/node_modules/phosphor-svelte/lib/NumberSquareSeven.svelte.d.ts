import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareSevenIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareSeven color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareSeven: Component<IconComponentProps, {}, "">;
type NumberSquareSeven = ReturnType<typeof NumberSquareSeven>;
export default NumberSquareSeven;
