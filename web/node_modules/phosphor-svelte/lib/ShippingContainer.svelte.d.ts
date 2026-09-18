import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShippingContainerIcon` instead.
 *
 * @example
 * ```svelte
 * <ShippingContainer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ShippingContainer: Component<IconComponentProps, {}, "">;
type ShippingContainer = ReturnType<typeof ShippingContainer>;
export default ShippingContainer;
