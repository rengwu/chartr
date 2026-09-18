import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShoppingCartSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <ShoppingCartSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShoppingCartSimple: Component<IconComponentProps, {}, "">;
type ShoppingCartSimple = ReturnType<typeof ShoppingCartSimple>;
export default ShoppingCartSimple;
