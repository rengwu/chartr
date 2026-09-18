import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberEightIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberEight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberEight: Component<IconComponentProps, {}, "">;
type NumberEight = ReturnType<typeof NumberEight>;
export default NumberEight;
