import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BeerBottleIcon` instead.
 *
 * @example
 * ```svelte
 * <BeerBottle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BeerBottle: Component<IconComponentProps, {}, "">;
type BeerBottle = ReturnType<typeof BeerBottle>;
export default BeerBottle;
