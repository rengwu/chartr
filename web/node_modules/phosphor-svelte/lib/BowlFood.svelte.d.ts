import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BowlFoodIcon` instead.
 *
 * @example
 * ```svelte
 * <BowlFood color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BowlFood: Component<IconComponentProps, {}, "">;
type BowlFood = ReturnType<typeof BowlFood>;
export default BowlFood;
