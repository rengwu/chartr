import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSquareFourIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSquareFour color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSquareFour: Component<IconComponentProps, {}, "">;
type NumberSquareFour = ReturnType<typeof NumberSquareFour>;
export default NumberSquareFour;
