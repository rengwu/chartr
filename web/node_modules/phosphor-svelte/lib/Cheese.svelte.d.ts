import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheeseIcon` instead.
 *
 * @example
 * ```svelte
 * <Cheese color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cheese: Component<IconComponentProps, {}, "">;
type Cheese = ReturnType<typeof Cheese>;
export default Cheese;
