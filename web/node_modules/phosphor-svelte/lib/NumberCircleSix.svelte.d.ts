import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberCircleSixIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberCircleSix color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberCircleSix: Component<IconComponentProps, {}, "">;
type NumberCircleSix = ReturnType<typeof NumberCircleSix>;
export default NumberCircleSix;
