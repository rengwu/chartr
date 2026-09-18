import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BabyCarriageIcon` instead.
 *
 * @example
 * ```svelte
 * <BabyCarriage color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BabyCarriage: Component<IconComponentProps, {}, "">;
type BabyCarriage = ReturnType<typeof BabyCarriage>;
export default BabyCarriage;
