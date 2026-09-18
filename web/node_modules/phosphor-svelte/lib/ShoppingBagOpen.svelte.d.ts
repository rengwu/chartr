import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShoppingBagOpenIcon` instead.
 *
 * @example
 * ```svelte
 * <ShoppingBagOpen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShoppingBagOpen: Component<IconComponentProps, {}, "">;
type ShoppingBagOpen = ReturnType<typeof ShoppingBagOpen>;
export default ShoppingBagOpen;
