import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleEightIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleEight color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleEight: Component<IconComponentProps, {}, "">;
type NumberCircleEight = ReturnType<typeof NumberCircleEight>;
export default NumberCircleEight;
