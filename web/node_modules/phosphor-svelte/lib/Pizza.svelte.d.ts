import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PizzaIcon` instead.
 *
 * @example
 * ```svelte
 * <Pizza color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pizza: Component<IconComponentProps, {}, "">;
type Pizza = ReturnType<typeof Pizza>;
export default Pizza;
