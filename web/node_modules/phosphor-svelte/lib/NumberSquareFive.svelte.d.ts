import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareFiveIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareFive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareFive: Component<IconComponentProps, {}, "">;
type NumberSquareFive = ReturnType<typeof NumberSquareFive>;
export default NumberSquareFive;
