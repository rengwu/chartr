import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleFiveIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleFive color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleFive: Component<IconComponentProps, {}, "">;
type NumberCircleFive = ReturnType<typeof NumberCircleFive>;
export default NumberCircleFive;
