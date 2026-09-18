import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BatteryVerticalMediumIcon` instead.
 *
 * @example
 * ```svelte
 * <BatteryVerticalMedium color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BatteryVerticalMedium: Component<IconComponentProps, {}, "">;
type BatteryVerticalMedium = ReturnType<typeof BatteryVerticalMedium>;
export default BatteryVerticalMedium;
