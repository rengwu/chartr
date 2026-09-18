import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShoppingBagIcon` instead.
 *
 * @example
 * ```svelte
 * <ShoppingBag color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShoppingBag: Component<IconComponentProps, {}, "">;
type ShoppingBag = ReturnType<typeof ShoppingBag>;
export default ShoppingBag;
