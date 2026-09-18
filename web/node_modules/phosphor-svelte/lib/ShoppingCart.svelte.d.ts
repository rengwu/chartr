import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShoppingCartIcon` instead.
 *
 * @example
 * ```svelte
 * <ShoppingCart color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShoppingCart: Component<IconComponentProps, {}, "">;
type ShoppingCart = ReturnType<typeof ShoppingCart>;
export default ShoppingCart;
