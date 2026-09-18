import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareTwoIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareTwo color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareTwo: Component<IconComponentProps, {}, "">;
type NumberSquareTwo = ReturnType<typeof NumberSquareTwo>;
export default NumberSquareTwo;
