import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CoffeeBeanIcon` instead.
 *
 * @example
 * ```svelte
 * <CoffeeBean color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CoffeeBean: Component<IconComponentProps, {}, "">;
type CoffeeBean = ReturnType<typeof CoffeeBean>;
export default CoffeeBean;
