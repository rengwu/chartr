import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StorefrontIcon` instead.
 *
 * @example
 * ```svelte
 * <Storefront color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Storefront: Component<IconComponentProps, {}, "">;
type Storefront = ReturnType<typeof Storefront>;
export default Storefront;
