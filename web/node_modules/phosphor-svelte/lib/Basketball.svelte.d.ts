import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BasketballIcon` instead.
 *
 * @example
 * ```svelte
 * <Basketball color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Basketball: Component<IconComponentProps, {}, "">;
type Basketball = ReturnType<typeof Basketball>;
export default Basketball;
