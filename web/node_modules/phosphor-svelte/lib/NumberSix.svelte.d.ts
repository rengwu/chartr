import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NumberSixIcon` instead.
 *
 * @example
 * ```svelte
 * <NumberSix color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NumberSix: Component<IconComponentProps, {}, "">;
type NumberSix = ReturnType<typeof NumberSix>;
export default NumberSix;
