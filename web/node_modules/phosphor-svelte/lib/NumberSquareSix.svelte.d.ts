import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareSixIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareSix color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareSix: Component<IconComponentProps, {}, "">;
type NumberSquareSix = ReturnType<typeof NumberSquareSix>;
export default NumberSquareSix;
