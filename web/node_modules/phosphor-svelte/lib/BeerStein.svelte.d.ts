import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BeerSteinIcon` instead.
 *
 * @example
 * ```svelte
 * <BeerStein color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BeerStein: Component<IconComponentProps, {}, "">;
type BeerStein = ReturnType<typeof BeerStein>;
export default BeerStein;
